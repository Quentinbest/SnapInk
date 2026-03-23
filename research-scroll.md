# Scrolling Screenshot Feature — Bug Investigation Report

**Date:** 2026-03-18
**Branch:** `fix/annotation-tool-ux`
**Reported symptoms:** White screen takeover, cursor moves but cannot interact, Esc exits the entire app.

---

## Executive Summary

The scrolling screenshot feature is **scaffolded at the menu/hotkey/type level but has zero functional implementation**. Triggering it opens a full-screen capture overlay that renders as a blank white screen because (a) no background screenshot is taken for this mode, and (b) no UI branch exists for `mode === 'scrolling'`. Pressing Esc closes the only window, which causes Tauri to exit the entire process.

---

## Issue 1: No Background Screenshot Captured for Scrolling Mode

**File:** `app/src-tauri/src/lib.rs:73`

```rust
if mode == "area" || mode == "window" || mode == "screen" {
    // pre-capture screenshot ...
}
```

The pre-capture screenshot (frozen desktop background) is only taken for `"area"`, `"window"`, and `"screen"` modes. When `mode == "scrolling"`, this block is skipped entirely. The `CaptureStore.background` remains `None`.

**Effect:** On the frontend, `get_capture_background` returns `null`, so `screenshotData` is `null`. The `<img class="bg-screenshot">` element is not rendered (guarded by `{#if screenshotData}`). The overlay div has no background → **white screen**.

**Severity:** Critical — root cause of the white screen.

---

## Issue 2: No UI Branch for Scrolling Mode in Capture Overlay

**File:** `app/src/routes/capture/+page.svelte:179-260`

The template has exactly two mode branches:

```svelte
{#if mode === 'area'}
  <!-- area selection UI: crosshairs, dim overlay, selection rect, action bar -->
{/if}

{#if mode === 'window'}
  <!-- window highlight UI -->
{/if}
```

There is **no `{#if mode === 'scrolling'}` block**. When scrolling mode is active, neither branch matches. The overlay renders as an empty div with `cursor: crosshair` — nothing visible except the cursor.

**Effect:** User sees a blank screen with a crosshair cursor. No dim overlay, no crosshairs, no instructions, no interactive elements. **The user cannot do anything except move the cursor.**

**Severity:** Critical — the feature has no UI.

---

## Issue 3: Mouse Interactions Ignored for Scrolling Mode

**File:** `app/src/routes/capture/+page.svelte:75-85`

```typescript
function onMouseDown(e: MouseEvent) {
    if (e.button !== 0) return;
    if (overlayState === 'idle' && mode === 'area') {    // ← only 'area'
        // start drag ...
    }
    if (mode === 'window' && hoveredWindowId !== null) {  // ← only 'window'
        captureWindow(hoveredWindowId);
    }
    // No handler for mode === 'scrolling'
}
```

Click events are only handled for `area` and `window` modes. For `scrolling`, `onMouseDown` returns without doing anything. Same for `onMouseUp` — it only processes drags started by the area mode.

**Effect:** Clicking anywhere on the overlay does nothing. The user is completely stuck.

**Severity:** Critical — no interaction possible.

---

## Issue 4: Esc Closes Capture Window → App Exits Entirely

**File:** `app/src/routes/capture/+page.svelte:127-129`

```typescript
async function cancel() {
    await appWindow.close();
}
```

**File:** `app/src/routes/capture/+page.svelte:166`

```svelte
<svelte:window onkeydown={(e) => e.key === 'Escape' && cancel()} />
```

Pressing Esc calls `cancel()` which calls `appWindow.close()`. This **destroys** the capture window (unlike the editor, which uses `prevent_close` + `hide()`).

**File:** `app/src-tauri/src/lib.rs:278`

```rust
.run(tauri::generate_context!())
```

There is **no `RunEvent::ExitRequested` handler** on the `.run()` call. Tauri 2's default behavior is to exit the process when the last window is destroyed. Since the editor window is created on-demand (not at startup) and no other windows exist at first launch, closing the capture window means zero windows remain → Tauri exits.

**Note:** The `Info.plist` sets `LSUIElement = true` (menu bar agent), but this only tells macOS not to show a Dock icon. It does **not** prevent Tauri from exiting when all windows close — that requires explicitly handling `RunEvent::ExitRequested` with `event.prevent_exit()`.

**Effect:** Pressing Esc kills the entire app. The tray icon disappears. The user must relaunch SnapInk.

**Severity:** Critical — data loss risk if editor had unsaved work in a hidden window.

---

## Issue 5: Scroll Button in Action Bar Has No Click Handler

**File:** `app/src/routes/capture/+page.svelte:231`

```svelte
<button class="action-btn secondary">📜 Scroll</button>
```

This button appears in the area-mode action bar (after completing a region selection) but has **no `onclick` handler**. It's a dead button. Even if a user selected a region via area mode and then wanted to start scrolling from that selection, clicking "Scroll" does nothing.

**Severity:** Medium — the button is unreachable in scrolling mode anyway (since area mode UI doesn't render), but it's broken even from the area capture flow.

---

## Issue 6: No Backend IPC Commands for Scrolling

**File:** `app/src-tauri/src/lib.rs:255-277` (invoke_handler registration)

The planned IPC commands from the design docs were never implemented:

| Planned Command | Status |
|---|---|
| `scroll_capture_start` | Not implemented |
| `scroll_capture_frame` | Not implemented |
| `scroll_capture_stop` | Not implemented |

**File:** `app/src-tauri/src/capture.rs`

Contains `capture_fullscreen`, `capture_region`, `capture_window_by_id` — no scrolling-related functions. No image stitching logic exists anywhere in the Rust codebase.

**Severity:** Critical — the entire backend for this feature is missing.

---

## Issue 7: No Scrolling State in Frontend Store

**File:** `app/src/lib/stores.svelte.ts`

The store has no scrolling-specific state. Missing:

- Frame buffer (array of captured frame images)
- Scroll capture lifecycle state (idle / capturing / stitching / complete)
- Stitching progress indicator
- Current frame index / total scroll height
- Reference to the target window being scrolled

**Severity:** Critical — no state management for the capture lifecycle.

---

## Issue 8: Type System Supports Scrolling but Nothing Consumes It

**File:** `app/src/lib/types.ts:71`

```typescript
export type CaptureMode = 'area' | 'screen' | 'window' | 'scrolling' | 'ocr' | 'repeat';
```

The type exists and compiles, but no code path in the frontend or backend branches on `'scrolling'` to do anything meaningful. The mode flows through correctly (URL param → `getUrlMode()` → `mode` state) but is then ignored by every conditional.

**Severity:** Low — not a bug per se, but highlights the gap between scaffolding and implementation.

---

## Root Cause Chain (User-Reported Symptoms)

```
User triggers "Scrolling Capture" (menu or Ctrl+Shift+6)
    ↓
open_capture_window(app, "scrolling") called
    ↓
mode != "area" | "window" | "screen" → NO pre-capture screenshot taken
    ↓
Capture overlay window opens full-screen (frameless, always-on-top)
    ↓
Frontend loads, mode = 'scrolling'
    ↓
get_capture_background returns null → no background image → WHITE SCREEN
    ↓
Template: mode !== 'area' AND mode !== 'window' → NO UI renders
    ↓
User sees: white screen with crosshair cursor, can move but can't click anything
    ↓
User presses Esc → cancel() → appWindow.close()
    ↓
Last window destroyed + no ExitRequested handler → TAURI EXITS PROCESS
    ↓
Tray icon disappears, app is gone
```

---

## Summary of All Issues

| # | Issue | Location | Severity |
|---|---|---|---|
| 1 | No background screenshot for scrolling mode | `lib.rs:73` | Critical |
| 2 | No UI branch for scrolling mode | `+page.svelte:179-260` | Critical |
| 3 | Mouse events ignored for scrolling mode | `+page.svelte:75-85` | Critical |
| 4 | Esc closes last window → app exits | `+page.svelte:127` + `lib.rs:278` | Critical |
| 5 | Scroll button has no onclick handler | `+page.svelte:231` | Medium |
| 6 | No backend IPC commands for scrolling | `lib.rs`, `capture.rs` | Critical |
| 7 | No scrolling state in frontend store | `stores.svelte.ts` | Critical |
| 8 | Type scaffolding exists but is unused | `types.ts:71` | Low |

**Bottom line:** The scrolling screenshot feature was designed in detail (see `plan-v2.md`, `plan-ui.md`, `research-ssa.md`) but only the entry points were wired up (menu item, hotkey, type definitions). The actual capture flow — UI, backend commands, image stitching — was never built. The immediate user-facing bugs (white screen, frozen UI, app crash on Esc) are all consequences of this missing implementation.
