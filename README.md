# SnapInk

A macOS menu bar screenshot and annotation tool. Capture any region, window, or full screen, annotate with 8 drawing tools, then save to file, copy to clipboard, or pin as a floating overlay.

---

## Features

### Capture modes

| Mode | Shortcut | Description |
|---|---|---|
| Area | `Ctrl+Shift+4` | Click-drag to select any region |
| Screen | `Ctrl+Shift+3` | Full-screen capture, opens editor immediately |
| Window | `Ctrl+Shift+5` | Click any window to capture it |
| Scrolling | `Ctrl+Shift+6` | Select a region, auto-scrolls and stitches a long screenshot |
| OCR | `Ctrl+Shift+7` | Select a region to recognize text *(UI only, not yet implemented)* |
| Repeat Last | `Ctrl+Shift+R` | Repeat the previous capture |

### Annotation tools

| Tool | Key | Description |
|---|---|---|
| Rectangle | `R` | Outlined or filled rectangle |
| Ellipse | `O` | Outlined or filled ellipse |
| Line | `L` | Straight line with endpoint handles |
| Arrow | `A` | Arrow with filled, open, double, or no arrowhead |
| Pen | `P` | Freehand drawing |
| Blur | `B` | Pixelated blur to redact content |
| Text | `T` | Click to place a text label |
| Step | `N` | Numbered callout circles (auto-increments) |

### Export options

- **Save to file** — PNG or JPEG, to a configurable folder with date-based filenames
- **Copy to clipboard** — with optional Retina (2×) resolution
- **Pin** — float the annotated image as a transparent always-on-top overlay

---

## Requirements

- macOS 12 or later
- Screen Recording permission (requested on first capture)

---

## Development

All commands run from the `app/` directory.

```bash
# Install dependencies
npm install

# Development (Vite on :1420 + Tauri hot reload)
npm run tauri dev

# Type-check the frontend
npm run check
npm run check:watch

# Production build (.dmg)
npm run tauri build
```

### Run tests

```bash
cd app/src-tauri
cargo test --lib
```

---

## Architecture

```
macOS menu bar / global shortcuts
          ↓
   Rust backend (Tauri 2)
          ↕  invoke() / events
   Webview (SvelteKit + Svelte 5)
```

The app runs as a persistent menu bar agent (`LSUIElement = true`). Windows are created on demand and hidden on close rather than destroyed, so the process stays alive.

### Process flow: Capture → Edit → Export

1. Tray menu or hotkey → `open_capture_cmd` → spawns the `/capture` overlay window (full screen, decorations off, always on top)
2. Before the overlay appears, a background screenshot is taken and stored in `CaptureStore` so the overlay can render a frozen background instantly
3. User selects a region → `crop_and_store` crops and saves the result in `CaptureStore`
4. `open_editor_cmd` opens the editor window; it calls `consume_capture_result()` to drain the store
5. Editor renders the image on a Konva.js canvas; annotations are pushed into `appStore`
6. Save / Copy / Pin calls Rust export commands

### Scrolling capture flow

1. User selects a region in the capture overlay and clicks "Start Scrolling Capture"
2. `start_scroll_capture_cmd` stores the physical-pixel region, closes the overlay, and opens a small floating pill window
3. The pill calls `start_auto_scroll_capture_cmd`, which spawns a background thread that:
   - Posts `CGEventCreateScrollWheelEvent2` scroll events every 300 ms (auto-scrolls the content under the cursor)
   - Captures a frame of the stored region after each scroll
   - Deduplicates consecutive identical frames
   - Emits `scroll-frame-added` events to update the frame counter in the pill
4. User presses Space or clicks Stop → `stop_scroll_capture_cmd` sets an `AtomicBool` flag → loop exits → `scroll-capture-done` event fires
5. `stitch_scroll_frames` aligns and merges all frames using overlap detection → result stored → editor opens

### Frontend state

Svelte 5 runes (`$state`, `$derived`) in `stores.svelte.ts`:

| Field | Type | Description |
|---|---|---|
| `annotations` | `Annotation[]` | All shapes on the canvas |
| `activeTool` | `ToolType` | Currently selected tool |
| `activeColor` | `string` | Current stroke/fill color |
| `strokeWidth` | `number` | Stroke width in px |
| `captureImageData` | `string` | Base64 PNG of the captured image |
| `undoStack` / `redoStack` | `Annotation[][]` | History, max 100 levels |
| `stepCounter` | `number` | Auto-increment for Step tool |
| `settings` | `Settings` | Persisted app settings |

### Routes

| Route | Window | Description |
|---|---|---|
| `/` | `editor` | Main annotation editor |
| `/capture?mode=…` | `capture` | Full-screen overlay for region/window selection |
| `/scroll-control` | `scroll-control` | Floating pill: frame counter, Stop, Cancel |
| `/pin?id=…` | `pin-{id}` | Floating pinned image |
| `/settings` | `settings` | 3-tab settings panel |

### Backend modules

| File | Role |
|---|---|
| `lib.rs` | App entry point: tray menu, global shortcuts, IPC command registration, window lifecycle |
| `capture.rs` | `xcap` wrapper → base64 PNG; monitor/window enumeration |
| `capture_store.rs` | `CaptureStore` and `ScrollCaptureStore` — bridge between capture and editor |
| `scroll.rs` | Auto-scroll loop: CGEvent injection, frame capture, stop flag |
| `stitch.rs` | Overlap detection and vertical frame stitching |
| `export.rs` | base64 → PNG/JPEG file write; filename pattern expansion |
| `settings.rs` | JSON read/write to `$CONFIG_DIR/SnapInk/settings.json` |
| `pin.rs` | `PinStore(HashMap<id, base64>)` for floating pin windows |
| `types.rs` | Shared serde types (mirrored in `src/lib/types.ts`) |

### IPC commands

| Group | Command | Description |
|---|---|---|
| Capture | `get_monitors` | List connected monitors |
| | `get_windows` | List visible windows |
| | `capture_fullscreen` | Capture a full monitor |
| | `capture_region` | Capture a region (used by capture overlay) |
| | `capture_window_by_id` | Capture a specific window |
| Store | `get_capture_background` | Retrieve the pre-taken background screenshot |
| | `crop_and_store` | Crop background to selection and store as result |
| | `consume_capture_result` | Drain the pending capture (called by editor on mount) |
| | `store_capture_result` | Store an already-captured image as the result |
| Scroll | `start_scroll_capture_cmd` | Store region, close overlay, open pill |
| | `start_auto_scroll_capture_cmd` | Start the auto-scroll + capture loop |
| | `stop_scroll_capture_cmd` | Signal the loop to stop |
| | `scroll_capture_add_frame` | Manually add one frame (legacy) |
| | `stitch_scroll_frames` | Stitch captured frames into one image |
| | `scroll_capture_reset` | Clear all scroll state and close the pill |
| Export | `export_to_file` | Write base64 image to a file path |
| | `expand_filename` | Expand a filename pattern with date/time tokens |
| | `get_default_save_path` | Read the configured save path from settings |
| Settings | `get_settings` | Load settings from disk |
| | `save_settings` | Persist settings to disk |
| Window | `open_capture_cmd` | Open the capture overlay |
| | `open_editor_cmd` | Open (or show) the editor window |
| | `open_settings_cmd` | Open (or show) the settings window |
| Pin | `pin_image` | Create a floating pin window |
| | `get_pin_image` | Retrieve a pinned image by ID |
| | `remove_pin_image` | Close a pin window and remove its data |
| Clipboard | `read_clipboard_image` | Read an image from the system clipboard |

---

## Settings

Settings are stored at `~/Library/Application Support/SnapInk/settings.json`.

| Section | Key | Default | Description |
|---|---|---|---|
| `capture` | `defaultMode` | `"region"` | Default capture mode |
| | `showCursor` | `false` | Include cursor in captures |
| | `captureDelay` | `0` | Delay in seconds before capturing |
| | `playSoundOnCapture` | `false` | Play a shutter sound |
| `afterCapture` | — | `"open_editor"` | `"open_editor"` · `"copy_clipboard"` · `"save_file"` |
| `alsoCopyAfterAnnotating` | — | `true` | Copy to clipboard when saving from the editor |
| `output` | `savePath` | Desktop | Default folder for saved files |
| | `filenamePattern` | `SnapInk {YYYY-MM-DD} at {HH.mm.ss}` | Filename template |
| | `format` | `"png"` | `"png"` or `"jpeg"` |
| | `jpegQuality` | `85` | JPEG quality 1–100 |
| | `retinaClipboard` | `true` | Copy at full Retina resolution |
| `hotkeys` | — | See table above | Rebindable global shortcuts |
| `annotations` | `defaultColor` | `"#FF3B30"` | Default tool color |
| `ui` | `theme` | `"system"` | `"system"` · `"light"` · `"dark"` |
| | `showMenuBarIcon` | `true` | Show icon in the menu bar |
| | `launchAtLogin` | `false` | Start at login |

### Filename pattern tokens

| Token | Value |
|---|---|
| `{YYYY}` | 4-digit year |
| `{MM}` | 2-digit month |
| `{DD}` | 2-digit day |
| `{HH}` | 2-digit hour (24 h) |
| `{mm}` | 2-digit minute |
| `{ss}` | 2-digit second |

---

## Key conventions

- **Image data over IPC is always base64-encoded PNG** (`String`)
- **Rust types** in `types.rs` and **TypeScript types** in `src/lib/types.ts` are manually kept in sync
- All windows are opened via Rust (`open_*_cmd` commands) — never `window.open()` from the frontend
- Global hotkeys use `Ctrl+Shift+[3-7, R]` to avoid conflicts with macOS system shortcuts
- The editor window is hidden on close (not destroyed) so the process stays alive as a menu bar agent

---

## Known gaps

- **OCR** — UI entry point exists but the Vision framework integration is not yet implemented
- **Hotkey recorder** — settings shows current bindings but cannot interactively rebind them
- **Code signing / notarization** — not yet set up for distribution outside local development
