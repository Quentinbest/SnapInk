# TODOS

## Completed

### Rust Unit Tests for core modules
**Completed:** v0.1.0 (2026-03-24)
Added 12 new tests across `capture_store.rs`, `stitch.rs`, and `ocr.rs`. Total: 30 tests passing.

### Scroll capture frame cap (memory safety)
**Completed:** v0.1.0 (2026-03-24)
Added `MAX_FRAMES = 500` hard cap in `scroll.rs` with `scroll-frame-cap-reached` event.

### OCR toast notification
**Completed:** v0.1.0 (2026-03-24)
Added toast in `capture/+page.svelte` showing copied text preview on success, "No text found" on NO_TEXT.

### DRY capture_region helpers
**Completed:** v0.1.0 (2026-03-24)
Extracted `crop_and_encode()` shared helper in `capture.rs`.

### Consolidate ScrollTarget into ScrollCaptureStore
**Completed:** v0.1.0 (2026-03-24)
Moved `scroll_target` field into `ScrollCaptureStore`, removed standalone `ScrollTarget` struct.

---

## Hotkey recorder UI (settings panel)
**What:** Allow users to rebind global shortcuts interactively in the Settings panel.
**Why:** Currently shortcuts are fixed (Ctrl+Shift+3-7, R). With OCR (Ctrl+Shift+7) shipping, users with conflicting shortcuts have no workaround. Settings panel shows hotkeys read-only but cannot rebind.
**Pros:** Removes friction for users with conflicting system shortcuts; matches user expectation for a tool they'll use daily.
**Cons:** Requires key-intercept UI in the webview + round-trip to save new shortcuts in `settings.rs` + re-register with `tauri-plugin-global-shortcut`. Medium complexity.
**Context:** Settings panel is at `src/routes/settings/+page.svelte`. Shortcut registration is in `lib.rs` (`app.global_shortcut().on_shortcut(...)`). `settings.rs` already persists `HotkeyBinding` structs.
**Depends on:** Nothing blocking. Independent feature.

## Document CaptureRegion coordinate space
**What:** Add a doc comment to `CaptureRegion` in `types.rs` noting it stores physical pixel coordinates (Svelte multiplies by `scaleFactor` before sending to Rust).
**Why:** The Retina coordinate conversion was a source of confusion during the panoramic capture design review. Without documentation, the next developer touching `CaptureRegion` may pass logical coordinates and get 2x wrong capture dimensions on Retina displays.
**Pros:** One-line comment prevents a class of Retina display bugs.
**Cons:** None.
**Context:** `CaptureRegion` is in `app/src-tauri/src/types.rs`. Physical pixel coordinates confirmed by `capture/+page.svelte:170` which multiplies by `scaleFactor`. The `capture_region_direct()` function divides by scale to convert back to points for `CGWindowListCreateImage`.
**Depends on:** Nothing blocking. Independent.

## Panoramic loop catch_unwind
**What:** Wrap the `run_panoramic_loop` thread body in `std::panic::catch_unwind` and emit `scroll-capture-error` on panic.
**Why:** If the polling thread panics (e.g., mutex poison), the scroll-control pill UI hangs in "scrolling" state forever with no feedback. The user must force-quit.
**Pros:** Converts a silent hang into a visible error message.
**Cons:** Minimal — `catch_unwind` is ~5 lines and zero runtime cost on non-panic paths.
**Context:** `run_panoramic_loop` is in `app/src-tauri/src/scroll.rs`. The thread is spawned in `lib.rs:start_panoramic_capture_cmd`. The pill UI listens for `scroll-capture-error` events and displays them.
**Depends on:** Nothing blocking. Independent.
