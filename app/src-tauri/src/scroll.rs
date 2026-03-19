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
}

#[cfg(target_os = "macos")]
#[link(name = "CoreFoundation", kind = "framework")]
extern "C" {
    fn CFRelease(cf: *const std::ffi::c_void);
}

/// Post a single scroll-down event (3 lines) to the window under the cursor.
/// No-op on non-macOS platforms.
#[cfg(target_os = "macos")]
fn post_scroll_down() {
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
            CGEventPost(0, event); // kCGHIDEventTap = 0
            CFRelease(event as *const _);
        }
    }
}

#[cfg(not(target_os = "macos"))]
fn post_scroll_down() {}

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
pub fn run_capture_loop(app: tauri::AppHandle, stop: Arc<AtomicBool>, interval_ms: u64) {
    use crate::capture_store::ScrollCaptureStore;

    // Confirm the loop is alive so the frontend can detect if the command succeeded
    // but the loop never started (e.g., immediate panic).
    let _ = app.emit("scroll-loop-started", ());

    loop {
        if stop.load(Ordering::Relaxed) {
            break;
        }

        post_scroll_down();

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
        post_scroll_down();
    }
}
