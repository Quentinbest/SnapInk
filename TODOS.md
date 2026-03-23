# TODOS

## Rust Unit Tests for core modules
**What:** Add `#[cfg(test)]` modules to `crop_and_store`, `stitch.rs`, and `ocr.rs`.
**Why:** No tests means edge-case bugs are caught at runtime. `crop_and_store` and `stitch.rs` have both caused production bugs (see fix commits for overlap detection). `cargo test` works today with zero new infrastructure.
**Pros:** Regression protection; catches zero-dimension crop, overlap edge cases, base64 decode failures at build time.
**Cons:** Small time investment per module (~30 min CC each).
**Context:** `ocr.rs` (new in OCR PR) will include the first test module as part of that PR. Follow-up: extend to `crop_and_store` and `stitch::is_duplicate` / `stitch::stitch_frames`.
**Depends on:** Nothing. Start with `cargo test` in `app/src-tauri/`.

## Hotkey recorder UI (settings panel)
**What:** Allow users to rebind global shortcuts interactively in the Settings panel.
**Why:** Currently shortcuts are fixed (Ctrl+Shift+3-7, R). With OCR (Ctrl+Shift+7) shipping, users with conflicting shortcuts have no workaround. Settings panel shows hotkeys read-only but cannot rebind.
**Pros:** Removes friction for users with conflicting system shortcuts; matches user expectation for a tool they'll use daily.
**Cons:** Requires key-intercept UI in the webview + round-trip to save new shortcuts in `settings.rs` + re-register with `tauri-plugin-global-shortcut`. Medium complexity.
**Context:** Settings panel is at `src/routes/settings/+page.svelte`. Shortcut registration is in `lib.rs` (`app.global_shortcut().on_shortcut(...)`). `settings.rs` already persists `HotkeyBinding` structs.
**Depends on:** Nothing blocking. Independent feature.
