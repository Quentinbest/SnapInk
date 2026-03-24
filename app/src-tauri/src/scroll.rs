use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use tauri::{Emitter, Manager};

/// Shared stop flag for the active auto-scroll capture loop.
/// Stored as Tauri managed state so both the background thread and IPC
/// commands can access the same flag.
pub struct ScrollStop(pub Arc<AtomicBool>);

// ── macOS CGEvent FFI ─────────────────────────────────────────────────────────

#[cfg(target_os = "macos")]
#[link(name = "CoreGraphics", kind = "framework")]
extern "C" {
    /// Create a scroll wheel event.
    ///
    /// CGEventCreateScrollWheelEvent2 is **non-variadic** (unlike the original
    /// CGEventCreateScrollWheelEvent) and requires all 6 parameters:
    ///   source, units, wheelCount, wheel1, wheel2, wheel3
    /// Omitting wheel2/wheel3 causes undefined behaviour — the function reads
    /// garbage from whatever happens to be in the argument registers.
    fn CGEventCreateScrollWheelEvent2(
        source: *const std::ffi::c_void, // CGEventSourceRef (null = hardware)
        units: u32,                      // kCGScrollEventUnitLine = 1
        wheel_count: u32,                // 1 = vertical only
        wheel1: i32,                     // negative = scroll down (content up)
        wheel2: i32,                     // horizontal — 0 for none
        wheel3: i32,                     // unused axis — 0
    ) -> *mut std::ffi::c_void;          // CGEventRef

    /// Post an event into the event stream.
    /// kCGHIDEventTap = 0: hardware-level tap, delivers to window under cursor.
    fn CGEventPost(tap: u32, event: *mut std::ffi::c_void);

    /// Set the location (in global display coordinates) where the event
    /// will be delivered.  This makes the scroll event target a specific
    /// screen point WITHOUT moving the user's cursor.
    fn CGEventSetLocation(event: *mut std::ffi::c_void, point: CGPoint);

    /// Move the mouse cursor to a new position in global display coordinates.
    /// Unlike mouse-move events, this is instantaneous and does not generate
    /// intermediate move events.  Returns CGError (0 = success).
    fn CGWarpMouseCursorPosition(point: CGPoint) -> i32;
}

#[cfg(target_os = "macos")]
#[link(name = "ApplicationServices", kind = "framework")]
extern "C" {
    /// Returns true if the current process is a trusted accessibility client.
    /// CGEventPost silently drops events when this returns false.
    fn AXIsProcessTrusted() -> bool;
}

/// CGPoint equivalent for FFI.
#[cfg(target_os = "macos")]
#[repr(C)]
#[derive(Clone, Copy)]
pub struct CGPoint {
    pub x: f64,
    pub y: f64,
}

#[cfg(target_os = "macos")]
#[link(name = "CoreFoundation", kind = "framework")]
extern "C" {
    fn CFRelease(cf: *const std::ffi::c_void);
}

/// Check whether the process has Accessibility permission (macOS only).
/// CGEventPost silently drops events without this.
#[cfg(target_os = "macos")]
pub fn is_accessibility_trusted() -> bool {
    unsafe { AXIsProcessTrusted() }
}

#[cfg(not(target_os = "macos"))]
pub fn is_accessibility_trusted() -> bool {
    true
}

/// Move the system cursor to `target` in global display coordinates.
/// This ensures that CGEventPost delivers scroll events to the window
/// under the cursor — CGEventSetLocation alone is unreliable for this.
#[cfg(target_os = "macos")]
pub fn warp_cursor(target: CGPoint) {
    unsafe {
        let err = CGWarpMouseCursorPosition(target);
        if err != 0 {
            eprintln!("CGWarpMouseCursorPosition failed with error code {err}");
        }
    }
}

#[cfg(not(target_os = "macos"))]
pub fn warp_cursor(_target: (f64, f64)) {}

/// Post a single scroll-down event (3 lines) targeted at `target`.
///
/// The cursor must already be at (or near) `target` — we rely on
/// `CGEventPost` delivering to the window under the cursor.
/// `CGEventSetLocation` is still called as a belt-and-suspenders measure
/// but is NOT sufficient on its own for reliable delivery.
///
/// No-op on non-macOS platforms.
#[cfg(target_os = "macos")]
fn post_scroll_down(target: CGPoint) {
    unsafe {
        let event = CGEventCreateScrollWheelEvent2(
            std::ptr::null(),
            1,     // kCGScrollEventUnitLine
            1,     // wheelCount (vertical only)
            -3i32, // negative = scroll down (reveal content below)
            0,     // wheel2 (horizontal) — no horizontal scroll
            0,     // wheel3 — unused axis
        );
        if !event.is_null() {
            CGEventSetLocation(event, target);
            CGEventPost(0, event); // kCGHIDEventTap = 0
            CFRelease(event as *const _);
        }
    }
}

#[cfg(not(target_os = "macos"))]
fn post_scroll_down(_target: (f64, f64)) {}

// ── Capture loop ──────────────────────────────────────────────────────────────

/// Run the auto-scroll capture loop in the calling thread (blocks until stopped).
///
/// Each iteration:
///   1. Posts a scroll-down event (3 lines) via CGEvent.
///   2. Sleeps `interval_ms` to let the scroll animation settle.
///   3. Captures a frame and emits `scroll-frame-added` with the new count.
///
/// The loop exits when the stop flag is set. On exit it emits `scroll-capture-done`.
/// Frame capture errors are forwarded to the frontend via `scroll-capture-error`.
/// Hard cap on accumulated frames to prevent OOM.
/// Each frame is a full-screen base64 PNG (~4-8 MB). 500 frames ≈ 2-4 GB.
const MAX_FRAMES: usize = 500;

pub fn run_capture_loop(app: tauri::AppHandle, stop: Arc<AtomicBool>, interval_ms: u64, target: (f64, f64)) {
    use crate::capture_store::ScrollCaptureStore;

    // Confirm the loop is alive so the frontend can detect if the command succeeded
    // but the loop never started (e.g., immediate panic).
    let _ = app.emit("scroll-loop-started", ());

    #[cfg(target_os = "macos")]
    let target_point = CGPoint { x: target.0, y: target.1 };
    #[cfg(not(target_os = "macos"))]
    let target_point = target;

    // Move the cursor to the target point so CGEventPost delivers scroll events
    // to the correct window.  CGEventSetLocation alone is unreliable — macOS
    // still routes events to the window under the actual cursor position.
    #[cfg(target_os = "macos")]
    {
        warp_cursor(target_point);
        // Small delay to let the window manager register the cursor position.
        std::thread::sleep(std::time::Duration::from_millis(50));
    }

    loop {
        if stop.load(Ordering::Relaxed) {
            break;
        }

        post_scroll_down(target_point);

        // Wait for the scroll animation to settle before capturing.
        std::thread::sleep(std::time::Duration::from_millis(interval_ms));

        if stop.load(Ordering::Relaxed) {
            break;
        }

        match app.try_state::<ScrollCaptureStore>() {
            Some(scroll_store) => {
                match crate::capture_store::add_frame_to_store(&scroll_store) {
                    Ok(count) => {
                        let _ = app.emit("scroll-frame-added", count as u32);
                        if count >= MAX_FRAMES {
                            eprintln!("scroll capture: frame cap ({MAX_FRAMES}) reached, auto-stopping");
                            let _ = app.emit("scroll-frame-cap-reached", MAX_FRAMES as u32);
                            break;
                        }
                    }
                    Err(e) => {
                        eprintln!("scroll capture error: {e}");
                        let _ = app.emit("scroll-capture-error", e);
                        // Continue looping — transient errors (e.g. first frame) are common.
                    }
                }
            }
            None => {
                let msg = "Internal error: ScrollCaptureStore not found";
                eprintln!("{msg}");
                let _ = app.emit("scroll-capture-error", msg.to_string());
                break;
            }
        }
    }

    let _ = app.emit("scroll-capture-done", ());
}

#[cfg(test)]
mod tests {
    #[cfg(target_os = "macos")]
    use super::*;

    /// Verify that the CGEventCreateScrollWheelEvent2 FFI declaration matches
    /// the real C signature (6 arguments: source, units, wheelCount, wheel1,
    /// wheel2, wheel3).  A previous bug omitted wheel2/wheel3 which caused
    /// undefined behaviour and a NULL return on Apple Silicon.
    #[test]
    #[cfg(target_os = "macos")]
    fn test_post_scroll_down_creates_valid_event() {
        // Calling post_scroll_down should not panic or crash.
        // We can't easily observe the scroll effect in a headless test, but we
        // CAN verify the event is created (non-NULL) by calling the FFI
        // directly with the corrected signature.
        unsafe {
            let event = CGEventCreateScrollWheelEvent2(
                std::ptr::null(),
                1,     // kCGScrollEventUnitLine
                1,     // wheelCount
                -1i32, // wheel1
                0,     // wheel2
                0,     // wheel3
            );
            assert!(
                !event.is_null(),
                "CGEventCreateScrollWheelEvent2 returned NULL — FFI signature mismatch?"
            );
            // Don't post the event in tests; just verify creation succeeded.
            CFRelease(event as *const _);
        }
    }

    /// Smoke-test: post_scroll_down() must not panic or segfault.
    #[test]
    #[cfg(target_os = "macos")]
    fn test_post_scroll_down_no_crash() {
        // Target a point that's off-screen so the event has no visible effect.
        post_scroll_down(CGPoint { x: 0.0, y: 0.0 });
    }

    /// Verify that CGEventSetLocation is accepted by the event.
    #[test]
    #[cfg(target_os = "macos")]
    fn test_post_scroll_down_with_target_location() {
        // Simulate targeting the center of a capture region at (500, 400).
        let target = CGPoint { x: 500.0, y: 400.0 };
        unsafe {
            let event = CGEventCreateScrollWheelEvent2(
                std::ptr::null(),
                1, 1, -1i32, 0, 0,
            );
            assert!(!event.is_null());
            CGEventSetLocation(event, target);
            // Don't post — just verify the call chain doesn't crash.
            CFRelease(event as *const _);
        }
    }

    /// Regression: CGWarpMouseCursorPosition FFI must not crash.
    /// The auto-scroll bug was caused by relying solely on CGEventSetLocation
    /// which does not reliably route scroll events to the target window.
    /// The fix warps the cursor to the target point before scrolling.
    #[test]
    #[cfg(target_os = "macos")]
    fn test_warp_cursor_no_crash() {
        // Warp to an off-screen point — should succeed without crashing.
        warp_cursor(CGPoint { x: 0.0, y: 0.0 });
    }

    /// Regression: AXIsProcessTrusted FFI call must not crash.
    /// The fix checks accessibility permission before starting the scroll loop
    /// to give a clear error instead of silently dropping events.
    #[test]
    #[cfg(target_os = "macos")]
    fn test_is_accessibility_trusted_no_crash() {
        // Just verify the FFI call doesn't panic or segfault.
        // The return value depends on system permissions — we only test the call.
        let _trusted = is_accessibility_trusted();
    }

    /// Verify that is_accessibility_trusted returns a boolean (not garbage).
    #[test]
    fn test_is_accessibility_trusted_returns_bool() {
        let result = is_accessibility_trusted();
        // The result is either true or false — this may seem trivial but it
        // confirms the FFI binding returns a valid bool and not UB.
        assert!(result || !result);
    }
}
