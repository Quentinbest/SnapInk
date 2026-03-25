# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## What is SnapInk

SnapInk is a macOS menu bar screenshot annotation tool built with **Tauri 2** (Rust backend) and **SvelteKit + Svelte 5** (frontend). It captures screen regions/windows, lets users annotate with 8 tools (rect, ellipse, line, arrow, pen, blur, text, step), and exports via file save, clipboard, or floating pin window.

## Development Commands

All commands run from the `app/` directory.

```bash
# Start dev server (Vite on :1420 + Tauri hot reload)
npm run tauri dev

# Type-check frontend
npm run check
npm run check:watch

# Production build (.dmg / platform bundle)
npm run tauri build

# Frontend only (no Tauri shell)
npm run dev
npm run build
```

### Tests

```bash
# Rust unit tests (30 tests across ocr, stitch, scroll, capture_store)
cd app/src-tauri && cargo test

# Frontend type-check only (no unit test framework)
npm run check
```

## Architecture

### Process Model

```
System (tray icon, global shortcuts)
        ↓ menu events / hotkeys
   Rust Backend (lib.rs)
        ↕ invoke() / IPC
   Webview (SvelteKit SPA)
```

The app runs as a single persistent process with multiple windows opened on demand. Windows share state through Tauri's managed state (`CaptureStore`, `ScrollCaptureStore`, `ScrollStop`, `PinStore`).

### Capture → Edit → Export Flow

1. **Tray menu or hotkey** triggers `open_capture_cmd` → spawns `/capture` overlay window
2. **CaptureStore** (`capture_store.rs`) holds a pre-captured full-screen background (base64 PNG) so the overlay renders instantly before the capture window appears on screen
3. User selects region → `crop_and_store` saves cropped result back into `CaptureStore`
4. `open_editor_cmd` opens the main editor window; it calls `consume_capture_result()` to drain the store
5. Editor (`+page.svelte`) renders the image via **AnnotationEngine** (Konva.js canvas wrapper) and pushes annotations into `appStore`
6. Save/copy/pin actions invoke Rust commands (`export_to_file`, clipboard plugin, `pin_image`)

### IPC Commands (28 total, registered in `lib.rs`)

| Group | Commands |
|---|---|
| Capture | `get_monitors`, `get_windows`, `capture_fullscreen`, `capture_region`, `capture_window_by_id`, `get_capture_background`, `crop_and_store`, `store_capture_result`, `consume_capture_result` |
| Scroll | `start_scroll_capture_cmd`, `start_panoramic_capture_cmd`, `stop_scroll_capture_cmd`, `scroll_capture_add_frame`, `stitch_scroll_frames`, `scroll_capture_reset` |
| OCR | `recognize_text`, `get_supported_ocr_languages` |
| Export | `export_to_file`, `expand_filename`, `get_default_save_path` |
| Settings | `get_settings`, `save_settings` |
| Window | `open_capture_cmd`, `open_editor_cmd`, `open_settings_cmd` |
| Pin | `pin_image`, `get_pin_image`, `remove_pin_image` |
| Clipboard | `read_clipboard_image` |

### Frontend State (`stores.svelte.ts`)

Uses Svelte 5 runes (`$state`). Key fields: `annotations[]`, `activeTool`, `activeColor`, `strokeWidth`, `captureImageData` (base64), `undoStack/redoStack` (max 100), `stepCounter`, `settings`.

### Routes

| Route | Purpose |
|---|---|
| `/` | Main annotation editor (780×440 base) |
| `/capture` | Full-screen overlay for region/window selection |
| `/scroll-control` | Floating pill: frame counter, Stop, Cancel |
| `/pin` | Floating always-on-top pin window |
| `/settings` | 3-tab settings panel (General, Shortcuts, Output) |

### Backend Modules

| File | Role |
|---|---|
| `lib.rs` | App orchestrator: tray menu, global shortcuts, all IPC command registration |
| `capture.rs` | `xcap` wrapper → base64 PNG; shared `crop_and_encode()` helper |
| `capture_store.rs` | `CaptureStore` and `ScrollCaptureStore` — bridges capture/scroll to editor |
| `scroll.rs` | Panoramic capture loop: polls screen changes, hash-based dedup, stop flag, 500-frame cap |
| `stitch.rs` | Overlap detection and vertical frame stitching |
| `ocr.rs` | macOS Vision OCR via `objc2-vision`; `recognize_text` and `get_supported_ocr_languages` IPC commands; auto-detect + explicit language support |
| `export.rs` | base64 → PNG/JPEG file write; `{YYYY}/{MM}/{DD}/{HH}/{mm}/{ss}` filename patterns |
| `settings.rs` | JSON I/O to `$CONFIG_DIR/SnapInk/settings.json` |
| `pin.rs` | `PinStore(HashMap<id, base64>)` for floating windows |
| `types.rs` | Shared serde types mirrored in `src/lib/types.ts` |

## Key Conventions

- **Image data is always base64-encoded PNG** passed over IPC as `String`
- **Rust types** in `types.rs` and **TypeScript types** in `src/lib/types.ts` must stay in sync manually
- All windows are opened via Rust (`open_*_cmd` commands) — never `window.open()` from the frontend
- Global hotkeys use `Ctrl+Shift+[3-7,R]` to avoid conflicts with macOS system shortcuts

## Known Gaps (not yet implemented)

- Hotkey recorder widget (settings shows hotkeys but can't rebind them interactively)
- Code signing / notarization for macOS distribution

## gstack

Use the `/browse` skill from gstack for all web browsing. Never use `mcp__claude-in-chrome__*` tools.

Available gstack skills: `/office-hours`, `/plan-ceo-review`, `/plan-eng-review`, `/plan-design-review`, `/design-consultation`, `/review`, `/ship`, `/land-and-deploy`, `/canary`, `/benchmark`, `/browse`, `/qa`, `/qa-only`, `/design-review`, `/setup-browser-cookies`, `/setup-deploy`, `/retro`, `/investigate`, `/document-release`, `/codex`, `/cso`, `/autoplan`, `/careful`, `/freeze`, `/guard`, `/unfreeze`, `/gstack-upgrade`.
