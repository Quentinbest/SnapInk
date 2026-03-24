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
