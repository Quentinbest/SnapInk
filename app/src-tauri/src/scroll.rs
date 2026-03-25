use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};

use tauri::{Emitter, Manager};

/// Shared stop flag for the active panoramic capture loop.
/// Stored as Tauri managed state so both the background thread and IPC
/// commands can access the same flag.
pub struct ScrollStop(pub Arc<AtomicBool>);

// ── Panoramic capture loop ───────────────────────────────────────────────────

/// Hard cap on accumulated frames to prevent OOM.
/// Each frame is a PNG-encoded region (~200KB-1MB). 500 frames ≈ 100-500 MB.
const MAX_FRAMES: usize = 500;

/// Run the panoramic capture loop in the calling thread (blocks until stopped).
///
/// The user scrolls naturally — this loop just polls for screen changes:
///   1. Captures the stored region via `capture_region_direct()`.
///   2. Hashes the PNG bytes and skips storage if unchanged (deduplication).
///   3. Sleeps for the remainder of the interval (elapsed-time-aware).
///
/// No scroll injection, no cursor warping, no Accessibility permission needed.
/// The loop exits when the stop flag is set. On exit it emits `scroll-capture-done`.
pub fn run_panoramic_loop(app: tauri::AppHandle, stop: Arc<AtomicBool>, interval_ms: u64) {
    use crate::capture_store::ScrollCaptureStore;

    let _ = app.emit("scroll-loop-started", ());

    let interval = Duration::from_millis(interval_ms);
    let mut last_hash: u64 = 0;

    loop {
        if stop.load(Ordering::Relaxed) {
            break;
        }

        let start = Instant::now();

        match app.try_state::<ScrollCaptureStore>() {
            Some(scroll_store) => {
                match crate::capture_store::add_frame_to_store(&scroll_store, &mut last_hash) {
                    Ok(count) => {
                        let _ = app.emit("scroll-frame-added", count as u32);
                        if count >= MAX_FRAMES {
                            eprintln!(
                                "scroll capture: frame cap ({MAX_FRAMES}) reached, auto-stopping"
                            );
                            let _ =
                                app.emit("scroll-frame-cap-reached", MAX_FRAMES as u32);
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

        if stop.load(Ordering::Relaxed) {
            break;
        }

        // Elapsed-time-aware scheduling: subtract capture duration from sleep.
        // Minimum 10ms sleep to yield to other threads.
        let elapsed = start.elapsed();
        let sleep_time = interval.saturating_sub(elapsed).max(Duration::from_millis(10));
        std::thread::sleep(sleep_time);
    }

    let _ = app.emit("scroll-capture-done", ());
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_scroll_stop_flag() {
        let stop = Arc::new(AtomicBool::new(false));
        assert!(!stop.load(Ordering::Relaxed));
        stop.store(true, Ordering::Relaxed);
        assert!(stop.load(Ordering::Relaxed));
    }

    #[test]
    fn test_max_frames_constant() {
        // Verify the frame cap is reasonable (not accidentally set to 0 or u64::MAX).
        assert!(MAX_FRAMES > 0);
        assert!(MAX_FRAMES <= 1000);
    }
}
