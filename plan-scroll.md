# Scrolling Screenshot — Implementation Plan

**Date:** 2026-03-18
**Based on:** `research-scroll.md` (bug investigation), `research-ssa.md` (algorithm research), `plan-ui.md` (UI spec)
**Approach:** Small incremental tasks, TDD where applicable, DRY, YAGNI. Each task ends with a commit.

---

## Architecture Decision: Manual Scroll Capture

After reviewing the research and reference tools (Shottr, Xnip), we adopt the **manual scroll** approach:

1. User selects a region on screen (reuse existing area selection)
2. User manually scrolls the target content (mouse wheel / trackpad)
3. The app captures frames at intervals during scrolling
4. Rust backend stitches frames using overlap detection
5. Stitched result opens in the editor

**Why manual scroll?** Programmatic scrolling requires accessibility permissions and fragile per-app scroll injection. Every reference macOS tool (Shottr, Xnip) uses manual scroll. This is YAGNI — we skip automated scrolling entirely.

**Why stitch in Rust?** The `image` crate is already a dependency. Pixel-level NCC overlap detection is CPU-intensive and benefits from native code. Base64 IPC is the existing pattern.

---

## Phase 1: Fix Critical Bugs (Issues 1, 4)

These are blocking bugs that affect the whole app, not just scrolling capture.

### Task 1.1 — Prevent app exit when last window closes

The app exits when the capture window is closed via Esc because Tauri exits on last window close by default. Fix: handle `RunEvent::ExitRequested` to keep the tray agent alive.

**Files to modify:**
- `app/src-tauri/src/lib.rs` — Change `.run(tauri::generate_context!())` to use a closure that calls `event.prevent_exit()`

**Code change:**
```rust
// Before (line 278):
.run(tauri::generate_context!())

// After:
.run(|_app, event| {
    if let tauri::RunEvent::ExitRequested { api, .. } = event {
        api.prevent_exit();
    }
})
```

**Testing:**
1. `npm run check` — type-check frontend (unchanged)
2. `cargo check` in `app/src-tauri/` — verify Rust compiles
3. Manual: launch app → open capture overlay → press Esc → verify tray icon remains and app is responsive

**Docs to consult:** [Tauri RunEvent docs](https://docs.rs/tauri/latest/tauri/enum.RunEvent.html)

**Commit:** `Fix app exit when closing last window — handle RunEvent::ExitRequested`

---

### Task 1.2 — Add background pre-capture for scrolling mode

The white screen happens because `open_capture_window` skips the screenshot for `mode == "scrolling"`.

**Files to modify:**
- `app/src-tauri/src/lib.rs:73` — Add `"scrolling"` to the mode match

**Code change:**
```rust
// Before:
if mode == "area" || mode == "window" || mode == "screen" {

// After:
if mode == "area" || mode == "window" || mode == "screen" || mode == "scrolling" {
```

**Testing:**
1. `cargo check` — verify compiles
2. Manual: trigger Scrolling Capture → verify the frozen desktop background renders (no white screen)

**Commit:** `Pre-capture background screenshot for scrolling mode`

---

## Phase 2: Scrolling Capture UI (Frontend)

### Task 2.1 — Add scrolling overlay state type and initial UI branch

Add the scrolling-specific overlay states and render a basic UI when `mode === 'scrolling'`.

**Files to modify:**
- `app/src/lib/types.ts:73` — Extend `OverlayState` to include scrolling states
- `app/src/routes/capture/+page.svelte` — Add `{#if mode === 'scrolling'}` block

**Type change:**
```typescript
// Before:
export type OverlayState = 'idle' | 'dragging' | 'complete';

// After:
export type OverlayState = 'idle' | 'dragging' | 'complete' | 'scroll-capturing' | 'scroll-done';
```

**Capture overlay changes:**
- When `mode === 'scrolling'`, reuse the area selection flow (crosshairs, drag-to-select, dim overlay) — this is already implemented for `mode === 'area'`
- Refactor: make the area selection UI render when `mode === 'area' || mode === 'scrolling'` instead of just `mode === 'area'`
- After selection is complete (`overlayState === 'complete'`), show a different action bar for scrolling mode with a "Start Scrolling Capture" button instead of the regular capture buttons

**Testing:**
1. `npm run check` — verify types
2. Manual: trigger Scrolling Capture → verify crosshairs appear, drag to select region, see scrolling-specific action bar

**Commit:** `Add scrolling mode UI branch with region selection`

---

### Task 2.2 — Implement scroll capture recording state

When the user clicks "Start Scrolling Capture", transition to `scroll-capturing` state. In this state:
- Hide the selection handles and action bar
- Show a floating status pill: "Capturing... N frames"
- Listen for the user's manual scrolling (no actual capture yet — just the UI state)
- Show a "Done" button to finish

**Files to modify:**
- `app/src/routes/capture/+page.svelte` — Add `scroll-capturing` state rendering, frame counter, done/cancel buttons

**New state variables (in capture page):**
```typescript
let scrollFrameCount = $state(0);
let scrollCaptureActive = $state(false);
```

**UI for `scroll-capturing` state:**
- Keep the selection rectangle visible (no handles)
- Floating pill at top of selection: `📜 Capturing... {scrollFrameCount} frames`
- Small "Done" and "Cancel" buttons below the pill
- Instruction at bottom: "Scroll the content now. Click Done when finished."

**Testing:**
1. `npm run check`
2. Manual: select region → click "Start Scrolling Capture" → see status pill → click Done/Cancel

**Commit:** `Add scroll capture recording UI state`

---

### Task 2.3 — Capture frames during scroll recording

Wire up actual frame capture during the `scroll-capturing` state. On a timer interval (every 300ms while active), call `capture_region` to grab the selected region and accumulate frames.

**Files to modify:**
- `app/src/routes/capture/+page.svelte` — Add timer-based capture loop, store frames as base64 strings

**Logic:**
```typescript
let scrollFrames: string[] = [];
let captureInterval: ReturnType<typeof setInterval> | null = null;

async function startScrollCapture() {
  overlayState = 'scroll-capturing';
  scrollFrames = [];
  scrollFrameCount = 0;

  // Capture initial frame
  await captureScrollFrame();

  // Start interval capture
  captureInterval = setInterval(captureScrollFrame, 300);
}

async function captureScrollFrame() {
  const scale = monitors[0].scaleFactor;
  const data = await invoke<string>('capture_region', {
    region: {
      x: Math.round(selection!.x * scale),
      y: Math.round(selection!.y * scale),
      width: Math.round(selection!.width * scale),
      height: Math.round(selection!.height * scale),
    },
    monitorIndex: 0,
  });
  scrollFrames.push(data);
  scrollFrameCount = scrollFrames.length;
}

function stopScrollCapture() {
  if (captureInterval) clearInterval(captureInterval);
  captureInterval = null;
  overlayState = 'scroll-done';
}
```

**Important:** The capture overlay window itself must NOT appear in the captured frames. The overlay is `always_on_top` and covers the screen. We need to either:
- **Option A:** Make the overlay transparent in the capture region so the underlying content is visible, and use `capture_region` which captures the screen (including what's behind our overlay if the overlay is transparent there). This won't work because `xcap` captures what's rendered on screen.
- **Option B:** Temporarily hide the overlay window before each capture, capture, then re-show. Too slow/flickery.
- **Option C:** Reduce the overlay to just a border frame around the selection (not covering the selection area). The dim overlay only covers the area OUTSIDE the selection (which is already how it works via `dimLeft/dimRight/dimTop/dimBottom`). The selection rectangle itself has `background: rgba(0, 122, 255, 0.04)` — nearly transparent. In `scroll-capturing` state, make the selection rect fully transparent and remove the border so xcap captures clean content through the "hole" in the dim overlay.

**Decision:** Option C is the right approach. The dim regions already leave a hole for the selection. We just need to make the selection rect border/background invisible during capture. Actually — re-reading the code, the selection rect has `pointer-events: none` and is just a visual overlay. The `xcap` library captures the screen bitmap from the compositor, NOT the webview content. The capture overlay window sits ON TOP of the target content, so `capture_region` would capture the overlay itself, including the dim areas.

**Revised approach:** During `scroll-capturing`, we need to **hide the entire capture overlay window** briefly for each frame capture. This is a fast Tauri IPC call:

```typescript
async function captureScrollFrame() {
  await appWindow.hide();
  // Small delay for compositor to update
  await new Promise(r => setTimeout(r, 50));

  const scale = monitors[0].scaleFactor;
  const data = await invoke<string>('capture_region', { ... });

  await appWindow.show();

  scrollFrames.push(data);
  scrollFrameCount = scrollFrames.length;
}
```

**Concern:** This will cause visible flickering. A better approach: capture from the pre-stored background? No — the background is a single static screenshot from before the overlay opened. We need LIVE content as the user scrolls.

**Better revised approach:** Instead of hiding/showing the overlay, we should **close the overlay entirely** during scroll capture and capture frames from a background Rust process. The user scrolls naturally without any overlay. A small floating indicator window (separate from the overlay) shows the frame count and "Done" button.

This requires rethinking the flow:
1. User selects region in overlay → clicks "Start Scrolling Capture"
2. Overlay closes, region coordinates are sent to Rust
3. Rust spawns a background capture loop (or frontend uses a small control window)
4. A tiny floating control window appears with "Done" / frame count
5. User scrolls content naturally
6. Clicking "Done" stops capture, stitches, opens editor

This is a significantly different architecture. Let's break it down properly in Tasks 2.3a-2.3c.

**Testing:** Deferred to sub-tasks below.

---

### Task 2.3a — Create scroll capture control window (Rust)

Add a new small floating window for the scroll capture controller. This replaces the full-screen overlay during the capture phase.

**Files to modify:**
- `app/src-tauri/src/lib.rs` — Add `open_scroll_control_window` function
- `app/src/routes/scroll-control/+page.svelte` — New route (small floating window)
- `app/src/routes/scroll-control/+page.ts` — Disable SSR

**Rust window creation:**
```rust
fn open_scroll_control_window(app: &tauri::AppHandle, region: CaptureRegion) {
    let url = format!(
        "/scroll-control?x={}&y={}&w={}&h={}",
        region.x, region.y, region.width, region.height
    );
    let _ = WebviewWindowBuilder::new(app, "scroll-control", WebviewUrl::App(url.into()))
        .title("Scroll Capture")
        .inner_size(220.0, 80.0)
        .resizable(false)
        .decorations(false)
        .always_on_top(true)
        .transparent(true)
        .build();
}
```

**Scroll control page** (`/scroll-control`): A pill-shaped floating widget with:
- Frame counter: "📜 3 frames"
- "Done" button
- "Cancel" button

**SvelteKit route setup:**
- `app/src/routes/scroll-control/+page.ts` — `export const ssr = false;`
- `app/src/routes/scroll-control/+page.svelte` — Small floating UI

**Testing:**
1. `npm run check`
2. `cargo check`
3. Manual: verify the control window opens as a small floating pill

**Commit:** `Add scroll capture control window route and Rust launcher`

---

### Task 2.3b — Implement frame capture loop in scroll control

The scroll control window runs a timer that captures the selected region repeatedly.

**Files to modify:**
- `app/src/routes/scroll-control/+page.svelte` — Add capture loop logic
- `app/src-tauri/src/capture_store.rs` — Add `ScrollCaptureStore` to hold frames

**New Rust state** (in `capture_store.rs`):
```rust
pub struct ScrollCaptureStore {
    pub frames: Mutex<Vec<String>>,
    pub region: Mutex<Option<CaptureRegion>>,
}
```

**New IPC commands** (in `capture_store.rs`):
```rust
#[tauri::command]
pub fn scroll_capture_add_frame(store: State<ScrollCaptureStore>) -> Result<usize, String>
// Captures the stored region, adds to frames vec, returns frame count

#[tauri::command]
pub fn scroll_capture_get_frames(store: State<ScrollCaptureStore>) -> Vec<String>
// Returns all captured frames

#[tauri::command]
pub fn scroll_capture_reset(store: State<ScrollCaptureStore>)
// Clears frames and region
```

**Why capture in Rust?** The Rust side can capture the screen region directly via `xcap` without the overlay interfering (the overlay is closed by then). The frontend just calls `scroll_capture_add_frame` on a timer.

**Frontend logic** (scroll-control page):
```typescript
onMount(() => {
  // Parse region from URL params
  // Start capture interval (every 400ms)
  // Each tick: invoke('scroll_capture_add_frame') → update frame count
});

function done() {
  clearInterval(interval);
  // Trigger stitch + open editor (Task 3.x)
}
```

**Capture interval:** 400ms is a good balance. At typical scroll speeds, this gives enough overlap between frames for the stitcher to work.

**Testing:**
1. `cargo check`
2. `npm run check`
3. Manual: start scroll capture → scroll some content → verify frame count increases → click Done → verify frames are stored

**Commit:** `Implement frame capture loop with ScrollCaptureStore`

---

### Task 2.3c — Wire capture overlay "Start" button to scroll control flow

Connect the capture overlay's scrolling mode to the new control window flow.

**Files to modify:**
- `app/src/routes/capture/+page.svelte` — Add `startScrollCapture()` function
- `app/src-tauri/src/lib.rs` — Add `start_scroll_capture_cmd` IPC command

**Flow:**
1. User selects region in capture overlay (scrolling mode)
2. Clicks "Start Scrolling Capture"
3. Frontend calls `invoke('start_scroll_capture_cmd', { region })` which:
   - Stores region in `ScrollCaptureStore`
   - Closes the capture overlay window
   - Opens the scroll control window
4. Scroll control window begins capturing

**Testing:**
1. `npm run check` + `cargo check`
2. Manual: full flow from Scrolling Capture menu → region select → Start → see control window → scroll → see frame count

**Commit:** `Wire scrolling capture flow from overlay to control window`

---

## Phase 3: Image Stitching (Rust Backend)

### Task 3.1 — Implement column-sampling overlap detection

Build the overlap detection algorithm. Use **column sampling** (not full NCC) for speed — sample 5 vertical columns, compare bottom strip of frame N with top portion of frame N+1.

**Files to create:**
- `app/src-tauri/src/stitch.rs` — New module for image stitching

**Algorithm:**
```rust
/// Find vertical overlap between two images using column sampling.
/// Returns the number of overlapping pixel rows, or 0 if no overlap found.
pub fn find_overlap(top: &DynamicImage, bottom: &DynamicImage) -> u32 {
    // 1. Both images must have the same width
    // 2. Extract bottom strip of `top` image (last `search_height` rows)
    //    search_height = min(top.height / 2, 200)
    // 3. For each candidate offset (0..search_height):
    //    - Compare 5 evenly-spaced columns between strip and bottom image
    //    - If column pixels match within tolerance → candidate overlap
    // 4. Return best matching overlap offset
}
```

**Column sampling detail:**
- Sample columns at positions: `[width*1/6, width*2/6, width*3/6, width*4/6, width*5/6]`
- For each column, compare a vertical slice of pixels
- Match threshold: > 95% of sampled pixels match (within RGB tolerance of 5 per channel)
- Search from largest overlap to smallest (greedy — first good match wins)

**Testing:**
- Unit test with synthetic images: create two overlapping images programmatically, verify `find_overlap` returns correct offset
- Edge case: no overlap (completely different images) → returns 0
- Edge case: identical images → returns full height

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use image::{RgbaImage, Rgba};

    #[test]
    fn test_find_overlap_exact() {
        // Create 100x200 image, split into two 100x120 frames with 40px overlap
        let full = RgbaImage::from_fn(100, 200, |x, y| {
            Rgba([(x % 256) as u8, (y % 256) as u8, 128, 255])
        });
        let top = DynamicImage::ImageRgba8(full.view(0, 0, 100, 120).to_image());
        let bottom = DynamicImage::ImageRgba8(full.view(0, 80, 100, 120).to_image());
        assert_eq!(find_overlap(&top, &bottom), 40);
    }

    #[test]
    fn test_find_overlap_none() {
        let a = DynamicImage::ImageRgba8(RgbaImage::from_pixel(100, 100, Rgba([255, 0, 0, 255])));
        let b = DynamicImage::ImageRgba8(RgbaImage::from_pixel(100, 100, Rgba([0, 0, 255, 255])));
        assert_eq!(find_overlap(&a, &b), 0);
    }
}
```

**Testing commands:**
```bash
cd app/src-tauri && cargo test stitch
```

**Commit:** `Implement column-sampling overlap detection with unit tests`

---

### Task 3.2 — Implement frame stitching

Given a `Vec<DynamicImage>` of frames with known overlaps, concatenate them vertically.

**Files to modify:**
- `app/src-tauri/src/stitch.rs` — Add `stitch_frames` function

**Function:**
```rust
/// Stitch multiple overlapping frames into a single tall image.
/// Returns the stitched image as base64 PNG.
pub fn stitch_frames(frames: Vec<DynamicImage>) -> Result<String, String> {
    if frames.is_empty() { return Err("No frames") }
    if frames.len() == 1 { return image_to_base64_png(&frames[0]) }

    // 1. Find overlaps between consecutive frames
    let overlaps: Vec<u32> = (0..frames.len()-1)
        .map(|i| find_overlap(&frames[i], &frames[i+1]))
        .collect();

    // 2. Calculate total height
    let total_height = frames[0].height()
        + frames[1..].iter().enumerate()
            .map(|(i, f)| f.height() - overlaps[i])
            .sum::<u32>();

    // 3. Create output image
    let width = frames[0].width();
    let mut result = RgbaImage::new(width, total_height);

    // 4. Copy frame pixels, skipping overlap regions
    let mut y_offset = 0u32;
    for (i, frame) in frames.iter().enumerate() {
        let skip = if i == 0 { 0 } else { overlaps[i-1] };
        let src = frame.to_rgba8();
        for y in skip..frame.height() {
            for x in 0..width {
                result.put_pixel(x, y_offset, *src.get_pixel(x, y));
            }
            y_offset += 1;
        }
    }

    image_to_base64_png(&DynamicImage::ImageRgba8(result))
}
```

**Testing:**
```rust
#[test]
fn test_stitch_two_frames() {
    // Create known overlapping frames, stitch, verify output dimensions
    let full = RgbaImage::from_fn(100, 200, |x, y| {
        Rgba([(x % 256) as u8, (y % 256) as u8, 128, 255])
    });
    let top = DynamicImage::ImageRgba8(full.view(0, 0, 100, 120).to_image());
    let bottom = DynamicImage::ImageRgba8(full.view(0, 80, 100, 120).to_image());

    let result = stitch_frames(vec![top, bottom]).unwrap();
    // Decode result and verify dimensions = 100x200
    let bytes = general_purpose::STANDARD.decode(&result).unwrap();
    let img = image::load_from_memory(&bytes).unwrap();
    assert_eq!(img.width(), 100);
    assert_eq!(img.height(), 200);
}

#[test]
fn test_stitch_single_frame() {
    let frame = DynamicImage::ImageRgba8(RgbaImage::from_pixel(50, 50, Rgba([0,0,0,255])));
    let result = stitch_frames(vec![frame]).unwrap();
    assert!(!result.is_empty());
}
```

**Testing commands:**
```bash
cd app/src-tauri && cargo test stitch
```

**Commit:** `Implement frame stitching with vertical concatenation and tests`

---

### Task 3.3 — Add stitch IPC command

Expose the stitcher to the frontend via a Tauri command.

**Files to modify:**
- `app/src-tauri/src/stitch.rs` — Add `#[tauri::command]` wrapper
- `app/src-tauri/src/lib.rs` — Register command, add `mod stitch;`

**Command:**
```rust
#[tauri::command]
pub fn stitch_scroll_frames(
    store: tauri::State<'_, ScrollCaptureStore>,
) -> Result<String, String> {
    let frames_b64 = store.frames.lock().unwrap().clone();
    if frames_b64.is_empty() {
        return Err("No frames captured".to_string());
    }

    let frames: Result<Vec<DynamicImage>, String> = frames_b64
        .iter()
        .map(|b64| {
            let bytes = general_purpose::STANDARD.decode(b64).map_err(|e| e.to_string())?;
            image::load_from_memory(&bytes).map_err(|e| e.to_string())
        })
        .collect();

    stitch_frames(frames?)
}
```

**Register in `lib.rs`:**
```rust
mod stitch;
// In invoke_handler:
stitch::stitch_scroll_frames,
```

**Testing:**
1. `cargo check` — compiles
2. `cargo test` — existing tests pass
3. Manual: will be tested end-to-end in Phase 4

**Commit:** `Add stitch_scroll_frames IPC command`

---

## Phase 4: End-to-End Flow

### Task 4.1 — Wire "Done" button to stitch and open editor

When the user clicks "Done" in the scroll control window, stitch all frames and open the editor.

**Files to modify:**
- `app/src/routes/scroll-control/+page.svelte` — Add `done()` function

**Logic:**
```typescript
async function done() {
  clearInterval(captureInterval);

  // Stitch frames in Rust
  const stitchedData = await invoke<string>('stitch_scroll_frames');

  // Store result for editor consumption
  await invoke('store_capture_result', { data: stitchedData });

  // Open editor
  await invoke('open_editor_cmd');

  // Clean up
  await invoke('scroll_capture_reset');

  // Close control window
  const win = getCurrentWindow();
  await win.close();
}
```

**Testing:**
1. `npm run check`
2. Manual: full end-to-end flow:
   - Trigger Scrolling Capture from tray menu
   - Select region over scrollable content (e.g., a long webpage)
   - Click "Start Scrolling Capture"
   - Scroll the content slowly
   - Click "Done"
   - Verify editor opens with a tall stitched image
   - Verify annotations can be drawn on the stitched image

**Commit:** `Wire scroll capture Done button to stitch and open editor`

---

### Task 4.2 — Handle cancel and edge cases

**Files to modify:**
- `app/src/routes/scroll-control/+page.svelte` — Cancel button, Esc key, minimum frame validation

**Cases to handle:**
- **Cancel:** Stop capture, clear frames, close control window
- **Esc key:** Same as cancel
- **< 2 frames:** Show toast "Need at least 2 frames. Keep scrolling." and don't stitch
- **Stitch error:** Show error toast, allow retry or cancel

**Testing:**
1. `npm run check`
2. Manual: test cancel flow, test with 0-1 frames, test Esc key

**Commit:** `Handle scroll capture cancel and edge cases`

---

### Task 4.3 — Add visual region indicator during scroll capture

While the scroll control window is active, draw a border around the capture region so the user knows what's being captured. Use a separate borderless transparent window.

**Files to modify:**
- `app/src-tauri/src/lib.rs` — Add region indicator window creation
- `app/src/routes/scroll-region/+page.svelte` — New route (transparent window with just a border)
- `app/src/routes/scroll-region/+page.ts` — `export const ssr = false;`

**Region indicator window:**
- Frameless, transparent, always-on-top, click-through (`ignore_cursor_events`)
- Positioned exactly over the selected region
- Renders just a dashed blue border (`2px dashed #007AFF`)
- No interaction — purely visual

**Testing:**
1. `npm run check` + `cargo check`
2. Manual: verify blue dashed border appears around the capture region during scroll capture

**Commit:** `Add visual region indicator during scroll capture`

---

## Phase 5: Polish

### Task 5.1 — Deduplicate identical consecutive frames

If the user pauses scrolling, the capture loop will produce identical frames. Detect and skip duplicates to save memory and avoid stitching artifacts.

**Files to modify:**
- `app/src-tauri/src/stitch.rs` — Add `is_duplicate_frame` function
- `app/src-tauri/src/capture_store.rs` — Use dedup check in `scroll_capture_add_frame`

**Algorithm:** Compare a quick hash/checksum of the frame data. Since frames are base64 strings, a simple string comparison of the last frame works. For performance, compare just the first 1000 bytes of base64 (covers the PNG header + first ~750 pixels which will differ if content changed).

**Testing:**
```rust
#[test]
fn test_duplicate_detection() {
    // Two identical images → detected as duplicate
    // Two different images → not duplicate
}
```

```bash
cd app/src-tauri && cargo test stitch
```

**Commit:** `Skip duplicate consecutive frames during scroll capture`

---

### Task 5.2 — Handle Scroll button from area capture mode

The existing "📜 Scroll" button in the area capture action bar (line 231 of `+page.svelte`) should transition to the scroll capture flow.

**Files to modify:**
- `app/src/routes/capture/+page.svelte:231` — Add `onclick={startScrollCapture}` to the Scroll button

**This reuses the same `startScrollCapture()` function** from Task 2.3c — the region is already selected, just pass it to the scroll capture flow.

**Testing:**
1. `npm run check`
2. Manual: do a regular area capture → instead of clicking Capture, click Scroll → verify it enters scrolling capture mode

**Commit:** `Wire Scroll button in area capture action bar`

---

### Task 5.3 — Style the scroll control window

Apply macOS-native styling to the floating control window.

**Files to modify:**
- `app/src/routes/scroll-control/+page.svelte` — Add polished styles

**Design (matching existing SnapInk style):**
- Pill-shaped container: `background: rgba(0, 0, 0, 0.72)`, `backdrop-filter: blur(20px)`, `border-radius: 9999px`
- White text, SF Mono font for frame count
- Compact layout: `📜 5 frames  [Done] [✕]`
- Matches the `coord-pill` / `dimension-pill` style from capture overlay

**Testing:**
1. Manual: verify the control window looks native and polished

**Commit:** `Style scroll capture control window`

---

## Task Dependency Graph

```
Phase 1 (bugs):
  1.1  Prevent app exit          ← independent
  1.2  Background pre-capture    ← independent

Phase 2 (UI):
  2.1  Scrolling UI branch       ← depends on 1.2
  2.2  Recording state UI        ← depends on 2.1
  2.3a Control window route      ← depends on 2.1
  2.3b Frame capture loop        ← depends on 2.3a
  2.3c Wire overlay → control    ← depends on 2.2, 2.3b

Phase 3 (stitching):
  3.1  Overlap detection         ← independent (pure algorithm)
  3.2  Frame stitching           ← depends on 3.1
  3.3  Stitch IPC command        ← depends on 3.2, 2.3b (needs ScrollCaptureStore)

Phase 4 (integration):
  4.1  Done → stitch → editor   ← depends on 3.3, 2.3c
  4.2  Cancel & edge cases       ← depends on 4.1
  4.3  Region indicator window   ← depends on 2.3a

Phase 5 (polish):
  5.1  Deduplicate frames        ← depends on 3.1
  5.2  Wire Scroll button        ← depends on 2.3c
  5.3  Style control window      ← depends on 2.3a
```

**Parallelizable:** Phase 3 (Tasks 3.1, 3.2) can be developed in parallel with Phase 2 since the stitching algorithm is pure Rust with no frontend dependencies.

---

## Files Modified Summary

| File | Tasks | Changes |
|---|---|---|
| `src-tauri/src/lib.rs` | 1.1, 1.2, 2.3a, 2.3c, 3.3, 4.3 | Exit handler, mode match, new commands, new windows |
| `src-tauri/src/capture_store.rs` | 2.3b | Add `ScrollCaptureStore`, frame capture commands |
| `src-tauri/src/stitch.rs` | 3.1, 3.2, 3.3, 5.1 | **New file** — overlap detection, stitching, IPC |
| `src/lib/types.ts` | 2.1 | Extend `OverlayState` |
| `src/routes/capture/+page.svelte` | 2.1, 2.2, 2.3c, 5.2 | Scrolling mode UI, start button handler |
| `src/routes/scroll-control/+page.svelte` | 2.3a, 2.3b, 4.1, 4.2, 5.3 | **New file** — control window |
| `src/routes/scroll-control/+page.ts` | 2.3a | **New file** — SSR disable |
| `src/routes/scroll-region/+page.svelte` | 4.3 | **New file** — region indicator |
| `src/routes/scroll-region/+page.ts` | 4.3 | **New file** — SSR disable |

---

## Testing Strategy

**No test framework exists** — the project uses only `svelte-check` for TypeScript verification. Testing approach:

| Layer | Method | Command |
|---|---|---|
| Rust unit tests | `#[cfg(test)]` in `stitch.rs` | `cd app/src-tauri && cargo test` |
| TypeScript types | `svelte-check` | `cd app && npm run check` |
| Rust compilation | `cargo check` | `cd app/src-tauri && cargo check` |
| Integration | Manual testing | `cd app && npm run tauri dev` |

**Rust unit tests** are the primary TDD artifact — the stitching algorithm is the most complex and testable part. Frontend changes are UI-driven and tested manually.

**Manual test script** (run after each phase):

1. **Phase 1:** Launch app → Scrolling Capture → verify no white screen → Esc → verify app stays alive
2. **Phase 2:** Scrolling Capture → select region → click Start → verify control window appears
3. **Phase 3:** `cargo test` — all stitch tests pass
4. **Phase 4:** Full flow → stitch → editor → annotate → save
5. **Phase 5:** Scroll with pauses → verify no duplicate frames → test Scroll button from area mode

---

## Estimated Commits

1. `Fix app exit when closing last window — handle RunEvent::ExitRequested`
2. `Pre-capture background screenshot for scrolling mode`
3. `Add scrolling mode UI branch with region selection`
4. `Add scroll capture recording UI state`
5. `Add scroll capture control window route and Rust launcher`
6. `Implement frame capture loop with ScrollCaptureStore`
7. `Wire scrolling capture flow from overlay to control window`
8. `Implement column-sampling overlap detection with unit tests`
9. `Implement frame stitching with vertical concatenation and tests`
10. `Add stitch_scroll_frames IPC command`
11. `Wire scroll capture Done button to stitch and open editor`
12. `Handle scroll capture cancel and edge cases`
13. `Add visual region indicator during scroll capture`
14. `Skip duplicate consecutive frames during scroll capture`
15. `Wire Scroll button in area capture action bar`
16. `Style scroll capture control window`
