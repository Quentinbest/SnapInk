# TODOS

## Rust Unit Tests for core modules
**What:** Add `#[cfg(test)]` modules to `capture_store.rs` (`crop_and_store`), `stitch.rs` (`is_duplicate` short strings, `stitch_frames` 3+ frames), and `ocr.rs` (oversized input >67MB, empty base64).
**Why:** Current coverage is 51% of code paths. `crop_and_store` has zero tests despite handling edge cases (no background, zero-dimension region, negative x/y). `stitch.rs` only tests 2-frame stitching. `ocr.rs` missing boundary tests.
**Pros:** Regression protection; catches zero-dimension crop, overlap edge cases, base64 decode failures at build time. ~12 new tests bring coverage to ~80%+.
**Cons:** Small time investment per module (~15 min CC each).
**Context:** `ocr.rs` already has 4 tests. `stitch.rs` has 10 tests but misses short-string prefix clamping and 3+ frame cumulative overlap. `capture_store.rs` needs a new `#[cfg(test)]` module. See eng review test plan at `~/.gstack/projects/Quentinbest-SnapInk/quentin-main-eng-review-test-plan-*.md`.
**Depends on:** Nothing. Start with `cargo test` in `app/src-tauri/`.

## Scroll capture frame cap (memory safety)
**What:** Add a 500-frame hard cap to `run_capture_loop` in `scroll.rs`. When reached, auto-stop and emit `scroll-capture-done` with a warning event.
**Why:** Each frame is a full-screen base64 PNG (~4-8 MB). 500 frames = ~2-4 GB. Without a cap, a forgotten scroll capture session can OOM the app. This was flagged as the only critical gap in the eng review.
**Pros:** Prevents OOM crashes. Users get their stitched result instead of a crash.
**Cons:** None meaningful — 500 frames is far beyond any realistic scroll capture.
**Context:** `run_capture_loop` in `scroll.rs:103`. Add `const MAX_FRAMES: usize = 500;` check after `scroll-frame-added` emit. Emit a `scroll-frame-cap-reached` event so the frontend can show a warning.
**Depends on:** Nothing.

## OCR toast notification
**What:** Add a brief toast/notification when OCR copies text to clipboard, so the user knows it worked.
**Why:** Currently OCR silently copies to clipboard with no visual feedback. User has no way to know if OCR succeeded or found no text without pasting somewhere.
**Pros:** Better UX — matches user expectation from other screenshot tools.
**Cons:** Requires a toast/notification component in the frontend. Could use Tauri's native notification API or a Svelte toast library.
**Context:** OCR flow is in `capture/+page.svelte` around the `recognize_text` invoke call. The `writeText` clipboard call is already there. Add toast after successful clipboard write. Show different message for "NO_TEXT" error.
**Depends on:** Nothing. Independent UX improvement.

## DRY capture_region helpers
**What:** Extract shared crop-to-PNG logic from `capture_region` and `capture_region_sync` in `capture.rs` into a single helper.
**Why:** Both functions duplicate the same crop → encode → base64 pipeline. Any fix to one must be manually mirrored to the other. DRY violation flagged in eng review.
**Pros:** Single source of truth for crop logic. Easier to maintain and test.
**Cons:** Minor refactor — low risk.
**Context:** `capture.rs` has `capture_region` (async, used by IPC) and `capture_region_sync` (sync, used by scroll capture). Both do: capture monitor → crop to region → PNG encode → base64. Extract the crop+encode+base64 portion into a shared `crop_to_base64(image, x, y, w, h) -> Result<String>`.
**Depends on:** Nothing.

## Consolidate ScrollTarget into ScrollCaptureStore
**What:** Move `ScrollTarget(Mutex<Option<(f64, f64)>>)` from its own managed state into `ScrollCaptureStore` as a field.
**Why:** Scroll capture now has 3 separate managed states: `ScrollStop`, `ScrollTarget`, `ScrollCaptureStore`. `ScrollTarget` is only used during scroll capture and should logically live with the other scroll state. Reduces cognitive overhead and `.manage()` calls.
**Pros:** Cleaner architecture, fewer managed states, easier to reason about scroll capture lifecycle.
**Cons:** Minor refactor — touch `lib.rs`, `scroll.rs`, `capture_store.rs`.
**Context:** `ScrollTarget` was added in the scroll-events-targeting fix (commit 309c111). `scroll_capture_reset` already resets `ScrollCaptureStore` — it should also clear the target. Folding them together makes reset complete by default.
**Depends on:** Nothing. Independent refactor.

## Hotkey recorder UI (settings panel)
**What:** Allow users to rebind global shortcuts interactively in the Settings panel.
**Why:** Currently shortcuts are fixed (Ctrl+Shift+3-7, R). With OCR (Ctrl+Shift+7) shipping, users with conflicting shortcuts have no workaround. Settings panel shows hotkeys read-only but cannot rebind.
**Pros:** Removes friction for users with conflicting system shortcuts; matches user expectation for a tool they'll use daily.
**Cons:** Requires key-intercept UI in the webview + round-trip to save new shortcuts in `settings.rs` + re-register with `tauri-plugin-global-shortcut`. Medium complexity.
**Context:** Settings panel is at `src/routes/settings/+page.svelte`. Shortcut registration is in `lib.rs` (`app.global_shortcut().on_shortcut(...)`). `settings.rs` already persists `HotkeyBinding` structs.
**Depends on:** Nothing blocking. Independent feature.
