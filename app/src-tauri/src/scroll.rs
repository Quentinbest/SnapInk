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
    /// Create a scroll wheel event (non-variadic declaration for wheel_count=1).
    /// Full C signature is variadic; we declare only the fixed args we use.
    fn CGEventCreateScrollWheelEvent2(
        source: *const std::ffi::c_void, // CGEventSourceRef (null = hardware)
        units: u32,                      // kCGScrollEventUnitLine = 1
        wheel_count: u32,                // 1 = vertical only
        wheel1: i32,                     // negative = scroll down (content up)
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
pub fn run_capture_loop(app: tauri::AppHandle, stop: Arc<AtomicBool>, interval_ms: u64) {
    use crate::capture_store::ScrollCaptureStore;

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

        if let Some(scroll_store) = app.try_state::<ScrollCaptureStore>() {
            match crate::capture_store::add_frame_to_store(&scroll_store) {
                Ok(count) => {
                    let _ = app.emit("scroll-frame-added", count as u32);
                }
                Err(e) => eprintln!("scroll capture error: {e}"),
            }
        }
    }

    let _ = app.emit("scroll-capture-done", ());
}
