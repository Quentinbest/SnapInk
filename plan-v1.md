# SnapInk — Engineering Implementation Plan v1.0

> From prototype to production: a complete technical blueprint for building SnapInk, a lightweight, fast, keyboard-driven screenshot annotation tool.

**Version**: 1.0
**Date**: 2026-03-14
**Status**: Approved for implementation

---

## Table of Contents

1. [Product Overview](#1-product-overview)
2. [Prototype Analysis](#2-prototype-analysis)
3. [System Architecture](#3-system-architecture)
4. [Technology Stack](#4-technology-stack)
5. [Core Functional Modules](#5-core-functional-modules)
6. [Internal Data Structures](#6-internal-data-structures)
7. [Key Algorithms](#7-key-algorithms)
8. [Code Examples](#8-code-examples)
9. [Performance Strategy](#9-performance-strategy)
10. [Development Roadmap](#10-development-roadmap)
11. [Testing Strategy](#11-testing-strategy)
12. [Packaging and Distribution](#12-packaging-and-distribution)
13. [Future Extensions](#13-future-extensions)

---

## 1. Product Overview

### 1.1 Purpose

SnapInk is a **menu bar screenshot annotation tool** targeting macOS as its primary platform, with Windows and Linux as secondary targets. It lives entirely in the system menu bar — no Dock icon, no persistent window — and activates on demand via global hotkeys. The end-to-end flow from hotkey press to annotated image in the clipboard must complete in under three seconds for the common case.

The name reflects the product's philosophy: screen-ink. Capture quickly, mark it up, ship it. Nothing more.

### 1.2 Target Users

| User Type | Primary Need |
|-----------|-------------|
| Developers | Bug reports, PR reviews, code walkthroughs |
| Designers | UI feedback, red-lining, design reviews |
| Product Managers | Feature specs, stakeholder walkthroughs |
| Technical Writers | Step-by-step documentation, tutorials |
| Support Teams | Reproducing issues, documenting user problems |

All personas share the same core need: **capture something, mark it up fast, send it**. They differ in which annotation types they use most.

### 1.3 Core User Scenarios

1. **Bug Report Flow**: Press `⌘⇧4` → drag region around the broken UI → draw a red arrow → add text "this button is misaligned" → `⌘C` to clipboard → paste into Jira/GitHub.
2. **Design Feedback**: Press `⌘⇧5` → click window → draw numbered steps on the mockup → save as PNG → attach to Figma comment.
3. **Tutorial Screenshot**: Press `⌘⇧4` → capture a dialog → blur the username field → add step counter badges → `⌘S` to save.
4. **Quick Reference**: Press `⌘⇧4` → capture a code snippet → pin as floating window for reference while writing.

### 1.4 Product Principles

**Speed-first**: The hotkey-to-editor latency must be under 200ms. Export must be under 100ms. No loading screens, no splash windows, no async spinners on critical paths.

**Invisible until needed**: Menu bar icon only. No Dock entry (`LSUIElement = true`). The app consumes under 30MB RAM when idle. No background telemetry, no network calls unless the user explicitly triggers a share action.

**Inline editing**: Annotations happen directly on the captured image inside the editor window. No separate property panels, no layer palettes. Tool options appear in context-sensitive popovers anchored to the toolbar.

**Non-destructive overlay model**: Annotations are stored as vector objects on a layer above the base image. The base image is never modified. Rasterization happens only at export time. Every annotation can be selected, moved, resized, and deleted at any point.

**macOS-native feel**: SF Pro typography, system blur materials, standard macOS window chrome (traffic-light buttons), system color picker integration, native file-save dialogs. The app should feel like it ships with the OS.

### 1.5 Differentiation

| Tool | Strengths | Weakness vs SnapInk |
|------|-----------|---------------------|
| CleanShot X | Polished, cloud share | macOS only, subscription, heavier |
| Shottr | Fast, small, great OCR | macOS only, closed source |
| Flameshot | Open source, cross-platform | Slow startup, heavy Qt dependency |
| Xnip | Good scrolling capture | macOS only, watermark on free tier |
| Greenshot | Windows polish | Windows only, C#/.NET runtime |

SnapInk's edge: **Tauri-based cross-platform binary under 15MB**, near-native capture speed, and a clean Svelte UI that matches each platform's native aesthetic.

---

## 2. Prototype Analysis

The Figma prototype (file `MhxyNHPAhPoADayWyNtXd4`) consists of four captured pages. Each maps to one or more engineering modules.

### 2.1 Screen Inventory and Module Mapping

| Figma Screen | States Shown | Engineering Module |
|---|---|---|
| Menu Bar Dropdown | Default, More submenu | `MenuBarController`, `HotkeyRegistry` |
| Region Selection Overlay | Idle, Dragging, Complete | `CaptureOverlay`, `RegionSelector` |
| Window Capture | Hover highlight, Window label | `WindowEnumerator`, `CaptureOverlay` |
| Annotation Editor (dark) | Full toolbar, annotations, popovers | `EditorWindow`, `AnnotationEngine`, `ToolController` |
| Annotation Editor (light) | Theme variant | `ThemeManager` |
| Tool Options Popovers | Arrow, Rectangle, Text, Blur | `ToolOptionsStore`, per-tool config |
| Object Selection State | Handles, bounding box, context bar | `SelectionManager`, `HandleRenderer` |
| Scrolling Capture | Setup, In Progress, Complete | `ScrollCapture`, `ImageStitcher` |
| Pin Image | Idle, Hover, Multiple stacked | `PinWindowManager` |
| OCR Flow | Region select, Result panel | `OCREngine` |
| Settings | General, Shortcuts, Output tabs | `SettingsStore`, `HotkeyRegistry` |

### 2.2 Annotation Toolbar

The toolbar rendered in the prototype contains 14 elements organized into four logical groups:

```
[ □  ○  ╱  ↗ ] [ ✎  ▦  A  ① ] [ ↺  ↻ ] [ 💾  📋  📌  ✕  ✓ ]
  Shapes         Draw/Special    History   Actions
```

**Group 1 — Shapes**: Rectangle, Ellipse, Line, Arrow
**Group 2 — Draw & Special**: Freehand Brush, Blur/Pixelate, Text, Step Counter
**Group 3 — History**: Undo, Redo
**Group 4 — Actions**: Save, Copy to Clipboard, Pin, Cancel, Done

Each tool has an **active state** (accent-colored background), a **hover state** (subtle highlight), and a **disabled state** (reduced opacity).

### 2.3 Tool Options Popovers

Four tool-specific option popovers appear anchored above the toolbar when the corresponding tool is active:

- **Arrow**: Stroke width slider (1–10px), Filled Head toggle, Angle Snap toggle
- **Rectangle**: Stroke width slider, Corner Radius slider (0–20px), Fill toggle
- **Text**: Font size slider (10–48px), Bold toggle, Background toggle
- **Blur/Pixelate**: Strength slider (1–20), Mode segmented control (Blur / Pixelate)

### 2.4 Color Palette Bar

Sits below the toolbar. Contains 8 preset swatches (red, orange, yellow, green, blue, purple, white, black) plus a custom gradient swatch that opens the system color picker. The selected color applies to the active tool's stroke and fill.

### 2.5 Region Selection UI

Three states observed in the prototype:
1. **Idle**: Fullscreen dark overlay, crosshair cursor, coordinate pill showing current cursor position
2. **Dragging**: Rubber-band selection with dashed border, dimension label `1024 × 768` updating live
3. **Complete**: Solid blue border with 8 resize handles, action bar with "Open in Editor" button

### 2.6 Key Interaction Patterns

- **Escape** cancels any active operation and dismisses the capture overlay
- **Enter / ✓** commits annotation and opens export options
- **⌘Z / ⌘⇧Z** undo/redo without leaving the editor
- **Clicking a tool** activates it; **clicking the canvas with no tool active** enters selection mode
- **Double-clicking a text annotation** re-enters inline text edit mode
- **Right-clicking canvas** shows a context menu (delete, duplicate, bring to front/back)

---

## 3. System Architecture

### 3.1 Application Structure

SnapInk uses a **Tauri v2** architecture: a Rust backend handles OS-level operations (screen capture, hotkeys, clipboard, file I/O), and a **Svelte** web frontend handles all UI rendering via the system WebView.

```
┌────────────────────────────────────────────────────────────────┐
│                        OS / Platform                           │
│   ScreenCaptureKit │ Win GraphicsCapture │ xdg-portal+PipeWire │
└───────────┬────────────────┬───────────────────┬───────────────┘
            │                │                   │
┌───────────▼────────────────▼───────────────────▼───────────────┐
│                    Rust Backend (Tauri)                         │
│  ┌──────────────┐ ┌──────────────┐ ┌────────────────────────┐  │
│  │ CaptureEngine│ │ HotkeyService│ │  ClipboardService      │  │
│  ├──────────────┤ ├──────────────┤ ├────────────────────────┤  │
│  │ WindowEnum   │ │ SettingsStore│ │  ExportService         │  │
│  ├──────────────┤ ├──────────────┤ ├────────────────────────┤  │
│  │ ScrollCapture│ │ OCREngine    │ │  PinWindowManager      │  │
│  └──────────────┘ └──────────────┘ └────────────────────────┘  │
│                  Tauri IPC Bridge (invoke / events)             │
└───────────┬────────────────────────────────────────────────────┘
            │
┌───────────▼────────────────────────────────────────────────────┐
│                  Svelte Frontend (WebView)                      │
│  ┌──────────────┐ ┌──────────────┐ ┌────────────────────────┐  │
│  │ EditorWindow │ │ToolController│ │  AnnotationEngine      │  │
│  ├──────────────┤ ├──────────────┤ ├────────────────────────┤  │
│  │ CanvasRenderer│ │SelectionMgr │ │  UndoRedoStack         │  │
│  ├──────────────┤ ├──────────────┤ ├────────────────────────┤  │
│  │ ColorPalette │ │ToolOptions   │ │  ThemeManager          │  │
│  └──────────────┘ └──────────────┘ └────────────────────────┘  │
└────────────────────────────────────────────────────────────────┘
```

### 3.2 Event Flow

```
Global Hotkey (OS)
      │
      ▼
HotkeyService (Rust)
      │  tauri::emit("capture:start", mode)
      ▼
CaptureOverlay (Svelte fullscreen window)
      │  user drags region / clicks window
      ▼
CaptureEngine (Rust)  ←── invoke("capture:region", {x, y, w, h})
      │  returns ImageData (base64 PNG)
      ▼
EditorWindow (Svelte window opens)
      │  receives imageData + metadata
      ▼
AnnotationEngine (Svelte + Konva)
      │  user annotates
      ▼
ExportService (Rust)  ←── invoke("export:clipboard" | "export:file", {blob})
      │
      ▼
OS Clipboard / File System
```

### 3.3 State Management

The frontend uses **Svelte stores** for reactive state. No external state library is needed.

```
appStore          — global app mode (idle | capturing | editing | pinned)
captureStore      — in-progress capture state (region, mode, display)
annotationStore   — document model (baseImage, annotations[], selection)
toolStore         — active tool + per-tool options
historyStore      — undo stack + redo stack
settingsStore     — persisted user preferences (hydrated from Rust on startup)
```

All stores derive from Svelte's `writable` / `derived` primitives. The `annotationStore` is the central source of truth for the editor.

### 3.4 IPC Contract (Tauri Commands)

```typescript
// Capture
invoke('capture_region', { x, y, width, height, scale }): Promise<string>  // base64 PNG
invoke('capture_window', { windowId }): Promise<string>
invoke('capture_fullscreen', { displayId }): Promise<string>
invoke('get_windows'): Promise<WindowInfo[]>
invoke('get_displays'): Promise<DisplayInfo[]>

// Export
invoke('export_to_clipboard', { imageData: string }): Promise<void>
invoke('export_to_file', { imageData: string, path: string }): Promise<void>
invoke('open_save_dialog', { defaultName: string }): Promise<string | null>

// Settings
invoke('get_settings'): Promise<Settings>
invoke('save_settings', { settings: Settings }): Promise<void>

// Pin
invoke('create_pin_window', { imageData: string, x, y, width, height }): Promise<number>
invoke('close_pin_window', { windowId: number }): Promise<void>

// OCR (macOS only)
invoke('ocr_region', { imageData: string }): Promise<string>

// Hotkeys
invoke('register_hotkeys', { bindings: HotkeyBinding[] }): Promise<void>
```

---

## 4. Technology Stack

### 4.1 Desktop Framework: Tauri v2

**Chosen over Electron** because:
- Final binary is 8–15MB (vs 150–200MB for Electron)
- WebView uses the OS native renderer (WKWebView on macOS, WebView2 on Windows, WebKitGTK on Linux)
- Rust backend has direct access to all platform APIs with zero overhead
- Memory footprint when idle: ~25MB vs ~80MB for Electron
- Security sandbox model prevents arbitrary file system access from the renderer

**Tradeoff**: The Rust learning curve is steeper than Node.js. Mitigate by keeping Rust surface minimal — only OS calls go in Rust; all business logic stays in TypeScript.

### 4.2 Frontend Framework: Svelte 5

**Chosen over React / Vue** because:
- Compiled output: no virtual DOM, no runtime overhead
- Runes API (`$state`, `$derived`) replaces Svelte stores for granular reactivity
- Resulting JS bundle is 40–60KB vs 150KB+ for React
- Component scoped CSS eliminates styling conflicts
- Native TypeScript support with zero configuration

### 4.3 Canvas Rendering: Konva.js

**Chosen over raw Canvas API / Fabric.js** because:
- Provides a **scene graph** with built-in z-ordering, hit testing, and event dispatch
- Objects are first-class (`Konva.Rect`, `Konva.Arrow`, `Konva.Text`, etc.)
- Selection handles (transformers) are built in (`Konva.Transformer`)
- Hardware-accelerated via `requestAnimationFrame` batching
- No external dependency on a full rendering framework (16KB gzipped)

**Layer structure in Konva**:
```
Stage
├── Layer: baseImageLayer    (static, only drawn once)
├── Layer: annotationLayer   (annotations, redrawn on change)
├── Layer: previewLayer      (active tool preview while drawing)
└── Layer: uiLayer           (selection handles, dimension pills)
```

### 4.4 Screenshot Capture

| Platform | API | Tauri Crate |
|----------|-----|-------------|
| macOS 14+ | `SCScreenshotManager` (ScreenCaptureKit) | `tauri-plugin-snapink-capture` (custom) |
| macOS 12–13 | `CGWindowListCreateImage` fallback | same plugin, runtime version check |
| Windows 10+ | `Windows.Graphics.Capture` API | same plugin, cfg target |
| Linux Wayland | `xdg-desktop-portal` + PipeWire | same plugin, D-Bus call |
| Linux X11 | `XGetImage` / `XShmGetImage` | same plugin, X11 fallback |

A single Tauri plugin (`tauri-plugin-snapink-capture`) abstracts platform differences. The plugin is built as a separate crate so it can be tested independently.

### 4.5 Storage

- **Settings**: JSON file at the platform config directory (`$HOME/Library/Application Support/SnapInk/settings.json` on macOS). Serialized with `serde_json`. Loaded synchronously on startup.
- **Annotation sessions**: Not persisted between sessions (in-memory only). A future version may add session recovery.
- **Recent captures**: Not stored. SnapInk is not an image manager.

### 4.6 Global Hotkeys

- **macOS**: `tauri-plugin-global-shortcut` (uses `CGEvent` tap + `RegisterEventHotKey`)
- **Windows**: Same plugin, uses `RegisterHotKey` Win32 API
- **Linux X11**: Same plugin, uses `XGrabKey`
- **Linux Wayland**: `xdg-desktop-portal` global shortcuts portal (compositors that support `org.freedesktop.portal.GlobalShortcuts`)

---

## 5. Core Functional Modules

### 5.1 Screenshot Capture Module

**Responsibilities**: Execute screen capture for all three modes (region, window, fullscreen), handle multi-monitor coordinate mapping, manage the Screen Recording permission prompt, return raw image data.

**Region selection sub-module**:
1. A borderless, transparent Tauri window (`decorations: false`, `transparent: true`) is created covering each monitor.
2. A full-screen snapshot is rendered as the window background to freeze content.
3. The Svelte overlay renders the crosshair cursor, coordinate pill (showing live pixel position), and rubber-band rectangle on mouse drag.
4. Keyboard modifier handling: Shift for square, Space to pan selection, Escape to cancel.
5. On mouse-up, the selected rect coordinates are sent via IPC to the Rust capture backend.

**Window detection**: On macOS, `SCShareableContent.getWithCompletionHandler` provides all on-screen windows with their titles, bounds, and owning process names. As the user moves the cursor, the frontend receives window metadata and draws a highlight border around the window under cursor.

**Multi-monitor**: Each display has its own Tauri overlay window. Coordinates are in global screen space (origin at top-left of primary display). The capture backend maps coordinates to the correct display.

**Edge cases**:
- HiDPI: logical coordinates from the frontend are multiplied by `window.devicePixelRatio` before passing to the capture backend.
- macOS Screen Recording permission: If the permission is not granted, Tauri shows a native alert directing the user to System Settings.

### 5.2 Annotation Engine

The annotation engine is a TypeScript module that owns the document model (base image + annotation array) and all mutation operations. It exposes a clean interface consumed by the Svelte editor components.

**Responsibilities**:
- Maintain the `AnnotationDocument` (base image + ordered annotation array)
- Dispatch mutations through the undo system
- Provide query methods (selection, bounding box, z-order)
- Render the annotation layer via Konva

**Internal structure**:
```
AnnotationEngine
├── document: AnnotationDocument
├── undoStack: UndoRedoStack
├── selection: SelectionManager
├── konvaStage: Konva.Stage
└── tools: Map<ToolType, Tool>
```

Each tool is a class implementing the `Tool` interface. The engine delegates all pointer events to the currently active tool.

### 5.3 Drawing System

**Object model**: Each annotation is an immutable-style record stored in the `annotations` array. Konva nodes are derived from these records and rebuilt when the record changes (reactive rendering via Svelte `$derived`).

**Hit detection strategy**:
1. Konva handles the primary hit detection via its built-in hit canvas (an off-screen canvas where each shape is painted with a unique color, enabling O(1) lookup).
2. For thin lines and arrows, Konva's hit tolerance is expanded to 10px.
3. Handle hit detection is managed by `Konva.Transformer` which automatically renders 8 resize handles and a rotation handle around selected shapes.

**Rendering strategy**: Konva batches all mutations and redraws only dirty layers on the next `requestAnimationFrame`. The base image layer is never redrawn after initial composition.

### 5.4 Undo / Redo System

Standard **Command Pattern** with two stacks. Every mutation to the annotation document goes through a command object.

```typescript
interface Command {
  execute(): void;
  undo(): void;
  description: string; // for debug/accessibility
}
```

The undo stack is capped at 100 commands to bound memory usage. When the cap is reached, the oldest commands are discarded (not undoable).

New commands always clear the redo stack.

### 5.5 Clipboard Integration

**Export path**: When the user presses `⌘C` or clicks the clipboard icon:
1. The frontend calls `konvaStage.toDataURL({ pixelRatio: window.devicePixelRatio })` to flatten all layers into a single data URL.
2. The data URL (PNG) is sent to the Rust backend via `invoke('export_to_clipboard', { imageData })`.
3. Rust decodes the base64, writes the PNG bytes to the OS clipboard.

On macOS, the Rust side uses `NSPasteboard` to write `image/png` data. On Windows, `CF_DIB`. On Linux, `wl-copy` or `xclip` subprocess.

### 5.6 Export System

**PNG export** (default): No re-encoding; Konva's `toDataURL` produces PNG. Lossless.

**JPEG export**: User can configure JPEG quality (60–100) in Settings. `toDataURL({ mimeType: 'image/jpeg', quality: 0.9 })`.

**Retina export**: The `pixelRatio` parameter controls output resolution. Default is `window.devicePixelRatio` (2× on Retina). User can override to 1× for smaller files.

**Filename templating**: Settings allow a filename pattern like `SnapInk_%Y%m%d_%H%M%S`. The Rust backend expands the pattern using `chrono` before writing the file.

### 5.7 Keyboard Shortcut System

Two layers of shortcuts:
1. **Global hotkeys** (system-wide, registered via Rust): Capture triggers (`⌘⇧3`, `⌘⇧4`, `⌘⇧5`, `⌘⇧6`).
2. **Local hotkeys** (editor-only, handled in Svelte): Tool selection, undo/redo, export actions.

Local shortcuts are registered via a `useKeyboard` Svelte action that listens on the editor window:

```typescript
const shortcuts: Record<string, () => void> = {
  'r': () => setTool('rect'),
  'e': () => setTool('ellipse'),
  'a': () => setTool('arrow'),
  't': () => setTool('text'),
  'b': () => setTool('brush'),
  'u': () => setTool('blur'),
  's': () => setTool('step'),
  'Meta+z': () => undo(),
  'Meta+Shift+z': () => redo(),
  'Meta+c': () => copyToClipboard(),
  'Meta+s': () => saveToFile(),
  'Escape': () => cancelOrDeselect(),
  'Delete': () => deleteSelected(),
  'Backspace': () => deleteSelected(),
};
```

---

## 6. Internal Data Structures

### 6.1 Annotation Document

```typescript
interface AnnotationDocument {
  id: string;                    // UUID, for session tracking
  baseImage: {
    dataUrl: string;             // original captured PNG as data URL
    width: number;               // logical pixels
    height: number;              // logical pixels
    scaleFactor: number;         // 1 or 2 (Retina)
    capturedAt: string;          // ISO timestamp
    sourceMode: CaptureMode;     // 'region' | 'window' | 'fullscreen' | 'scroll'
  };
  annotations: Annotation[];     // ordered array, index = z-order
  viewport: {
    zoom: number;                // 1.0 = 100%
    panX: number;
    panY: number;
  };
}
```

### 6.2 Annotation Object

```typescript
type AnnotationType =
  | 'rect' | 'ellipse' | 'line' | 'arrow'
  | 'text' | 'brush' | 'blur' | 'step';

interface BaseAnnotation {
  id: string;
  type: AnnotationType;
  style: AnnotationStyle;
  locked: boolean;
}

interface AnnotationStyle {
  strokeColor: string;           // hex, e.g. "#FF3B30"
  strokeWidth: number;           // logical pixels
  fillColor: string | null;      // null = no fill
  opacity: number;               // 0–1
}

interface RectAnnotation extends BaseAnnotation {
  type: 'rect';
  x: number; y: number;
  width: number; height: number;
  cornerRadius: number;
}

interface EllipseAnnotation extends BaseAnnotation {
  type: 'ellipse';
  cx: number; cy: number;        // center
  rx: number; ry: number;        // radii
}

interface ArrowAnnotation extends BaseAnnotation {
  type: 'arrow';
  x1: number; y1: number;        // tail
  x2: number; y2: number;        // head
  filledHead: boolean;
  angleSnap: boolean;
}

interface TextAnnotation extends BaseAnnotation {
  type: 'text';
  x: number; y: number;
  text: string;
  fontSize: number;
  bold: boolean;
  hasBackground: boolean;
}

interface BrushAnnotation extends BaseAnnotation {
  type: 'brush';
  points: number[];              // flat array: [x0, y0, x1, y1, ...]
  tension: number;               // Catmull-Rom smoothing (0–1)
}

interface BlurAnnotation extends BaseAnnotation {
  type: 'blur';
  x: number; y: number;
  width: number; height: number;
  strength: number;              // blur radius
  mode: 'gaussian' | 'pixelate';
}

interface StepAnnotation extends BaseAnnotation {
  type: 'step';
  cx: number; cy: number;        // center of badge
  stepNumber: number;            // auto-incremented
}
```

### 6.3 Undo History Storage

```typescript
interface UndoRedoStack {
  undoStack: Command[];          // most recent last
  redoStack: Command[];          // most recent first
  maxDepth: number;              // default: 100
}

// Concrete command examples
class AddAnnotationCommand implements Command {
  constructor(
    private engine: AnnotationEngine,
    private annotation: Annotation
  ) {}
  execute() { this.engine.addAnnotationDirect(this.annotation); }
  undo()    { this.engine.removeAnnotationDirect(this.annotation.id); }
  description = `Add ${this.annotation.type}`;
}

class MoveAnnotationCommand implements Command {
  constructor(
    private engine: AnnotationEngine,
    private id: string,
    private from: Point,
    private to: Point
  ) {}
  execute() { this.engine.moveAnnotationDirect(this.id, this.to); }
  undo()    { this.engine.moveAnnotationDirect(this.id, this.from); }
  description = `Move ${this.id}`;
}
```

### 6.4 Settings Schema

```typescript
interface Settings {
  version: number;               // schema version for migrations
  capture: {
    defaultMode: CaptureMode;
    showCursor: boolean;
    captureDelay: number;        // ms before capture (0–3000)
    soundEffect: boolean;
  };
  output: {
    savePath: string;            // default save directory
    filenamePattern: string;     // e.g. "SnapInk_%Y%m%d_%H%M%S"
    format: 'png' | 'jpeg' | 'webp';
    jpegQuality: number;         // 60–100
    retinaExport: boolean;       // 2× output on HiDPI
  };
  hotkeys: HotkeyBinding[];
  annotations: {
    defaultColor: string;
    defaultStrokeWidth: number;
    palette: string[];           // 8-color preset palette
  };
  ui: {
    theme: 'system' | 'light' | 'dark';
    toolbarPosition: 'bottom' | 'top';
  };
  afterCapture: 'open_editor' | 'copy_to_clipboard' | 'save_to_file';
}
```

---

## 7. Key Algorithms

### 7.1 Arrow Rendering

```typescript
function drawArrow(ctx: CanvasRenderingContext2D, x1: number, y1: number,
                   x2: number, y2: number, strokeWidth: number,
                   filledHead: boolean): void {
  const headLength = Math.max(strokeWidth * 4, 14);
  const headWidth  = headLength * 0.55;

  const dx = x2 - x1;
  const dy = y2 - y1;
  const angle = Math.atan2(dy, dx);

  // Shaft
  ctx.beginPath();
  ctx.moveTo(x1, y1);
  ctx.lineTo(x2, y2);
  ctx.lineWidth = strokeWidth;
  ctx.stroke();

  // Arrowhead
  const lx = x2 - Math.cos(angle - Math.PI / 6) * headLength;
  const ly = y2 - Math.sin(angle - Math.PI / 6) * headLength;
  const rx = x2 - Math.cos(angle + Math.PI / 6) * headLength;
  const ry = y2 - Math.sin(angle + Math.PI / 6) * headLength;

  ctx.beginPath();
  ctx.moveTo(x2, y2);
  ctx.lineTo(lx, ly);
  if (filledHead) {
    ctx.lineTo(rx, ry);
    ctx.closePath();
    ctx.fill();
  } else {
    ctx.stroke();
    ctx.moveTo(x2, y2);
    ctx.lineTo(rx, ry);
    ctx.stroke();
  }
}
```

**Angle snapping** (when Shift is held): Round `angle` to the nearest 45°:
```typescript
function snapAngle(angle: number): number {
  const snap = Math.PI / 4; // 45°
  return Math.round(angle / snap) * snap;
}
```

### 7.2 Blur Algorithm

When a blur annotation is created or resized, extract and process the region from the base image:

```typescript
function computeBlurRegion(
  baseImageData: ImageData,
  region: { x: number; y: number; width: number; height: number },
  strength: number,
  mode: 'gaussian' | 'pixelate'
): ImageData {
  const regionData = extractRegion(baseImageData, region);

  if (mode === 'pixelate') {
    return pixelate(regionData, strength); // block size = strength * 2
  } else {
    // Three-pass box blur approximates Gaussian
    let result = boxBlur(regionData, strength);
    result = boxBlur(result, strength);
    result = boxBlur(result, strength);
    return result;
  }
}

function pixelate(data: ImageData, blockSize: number): ImageData {
  const out = new ImageData(data.width, data.height);
  for (let y = 0; y < data.height; y += blockSize) {
    for (let x = 0; x < data.width; x += blockSize) {
      const [r, g, b, a] = averageBlock(data, x, y, blockSize);
      fillBlock(out, x, y, blockSize, r, g, b, a);
    }
  }
  return out;
}
```

The computed blur region is stored as a separate `offscreenCanvas` cached on the `BlurAnnotation` object. It is only recomputed when the region geometry changes, not on every render frame.

### 7.3 Freehand Smoothing

Raw pointer events produce jagged paths. Apply Catmull-Rom spline smoothing:

```typescript
function smoothPoints(points: number[], tension = 0.4): number[] {
  if (points.length < 8) return points; // need at least 4 points
  const smoothed: number[] = [points[0], points[1]];

  for (let i = 2; i < points.length - 2; i += 2) {
    const x0 = points[i - 2], y0 = points[i - 1];
    const x1 = points[i],     y1 = points[i + 1];
    const x2 = points[i + 2], y2 = points[i + 3];

    // Control points
    const cp1x = x1 - (x2 - x0) * tension / 2;
    const cp1y = y1 - (y2 - y0) * tension / 2;

    smoothed.push(cp1x, cp1y, x1, y1);
  }
  smoothed.push(points[points.length - 2], points[points.length - 1]);
  return smoothed;
}
```

Konva's `Line` with `tension` property handles this natively; the custom implementation is a fallback for the raw canvas path.

### 7.4 Step Counter Auto-Increment

The step counter tool auto-assigns the next available number:

```typescript
function nextStepNumber(annotations: Annotation[]): number {
  const used = annotations
    .filter(a => a.type === 'step')
    .map(a => (a as StepAnnotation).stepNumber);
  let n = 1;
  while (used.includes(n)) n++;
  return n;
}
```

When a step annotation is deleted, remaining numbers are **not** renumbered (preserving user intent). The user can manually reorder step numbers via the selection context menu in a future version.

### 7.5 Hit Testing for Selection

When the user clicks the canvas in selection mode (no active draw tool):

```typescript
function hitTest(point: Point, annotations: Annotation[]): string | null {
  // Test in reverse z-order (top-most first)
  for (let i = annotations.length - 1; i >= 0; i--) {
    const ann = annotations[i];
    if (hitTestAnnotation(point, ann)) return ann.id;
  }
  return null;
}

function hitTestAnnotation(point: Point, ann: Annotation): boolean {
  const tolerance = 8; // pixels
  switch (ann.type) {
    case 'rect':
      return pointNearRect(point, ann as RectAnnotation, tolerance);
    case 'arrow':
    case 'line':
      return pointNearSegment(point, ann as ArrowAnnotation, tolerance);
    case 'text':
      return pointInRect(point, getBoundingBox(ann));
    case 'blur':
    case 'step':
      return pointInRect(point, getBoundingBox(ann));
    default:
      return false;
  }
}
```

Konva's built-in hit canvas is used for performance in the actual implementation; the above is the fallback pure-TS path.

---

## 8. Code Examples

### 8.1 Screenshot Capture (Rust + Tauri)

```rust
// src-tauri/src/capture/macos.rs
use tauri::command;

#[command]
pub async fn capture_region(
    x: f64, y: f64,
    width: f64, height: f64,
    scale: f64,
) -> Result<String, String> {
    #[cfg(target_os = "macos")]
    {
        use screencapturekit::sc_screenshot_manager::SCScreenshotManager;
        use screencapturekit::sc_content_filter::SCContentFilter;
        use screencapturekit::sc_stream_configuration::SCStreamConfiguration;

        let filter = SCContentFilter::new_with_display_excluding_windows(
            get_display_for_rect(x, y, width, height)?,
            &[],
        );

        let mut config = SCStreamConfiguration::new();
        config.set_source_rect(CGRect::new(
            CGPoint::new(x * scale, y * scale),
            CGSize::new(width * scale, height * scale),
        ));
        config.set_scales_to_fit(false);

        let image = SCScreenshotManager::capture_image_synchronously(
            &filter,
            &config,
        ).map_err(|e| e.to_string())?;

        let png_data = cg_image_to_png(&image)?;
        Ok(base64::encode(&png_data))
    }
}
```

### 8.2 Konva Canvas Setup (Svelte)

```svelte
<!-- src/lib/editor/Canvas.svelte -->
<script lang="ts">
  import Konva from 'konva';
  import { onMount, onDestroy } from 'svelte';
  import { annotationStore, toolStore } from '$lib/stores';

  let containerEl: HTMLDivElement;
  let stage: Konva.Stage;
  let baseLayer: Konva.Layer;
  let annotationLayer: Konva.Layer;
  let previewLayer: Konva.Layer;
  let transformer: Konva.Transformer;

  export let document: AnnotationDocument;

  onMount(() => {
    stage = new Konva.Stage({
      container: containerEl,
      width: document.baseImage.width,
      height: document.baseImage.height,
    });

    // Layer 1: Static base image (never redrawn)
    baseLayer = new Konva.Layer({ listening: false });
    const baseImg = new Konva.Image({ x: 0, y: 0 });
    const img = new Image();
    img.onload = () => {
      baseImg.image(img);
      baseLayer.add(baseImg);
      baseLayer.draw();
    };
    img.src = document.baseImage.dataUrl;

    // Layer 2: Annotations
    annotationLayer = new Konva.Layer();
    transformer = new Konva.Transformer({
      rotateEnabled: false,
      boundBoxFunc: (oldBox, newBox) => newBox,
    });
    annotationLayer.add(transformer);

    // Layer 3: Drawing preview (active tool)
    previewLayer = new Konva.Layer({ listening: false });

    stage.add(baseLayer, annotationLayer, previewLayer);

    // Wire pointer events to active tool
    stage.on('mousedown', (e) => $toolStore.activeTool?.onMouseDown(e));
    stage.on('mousemove', (e) => $toolStore.activeTool?.onMouseMove(e));
    stage.on('mouseup',   (e) => $toolStore.activeTool?.onMouseUp(e));
  });

  // Reactive: rebuild annotation nodes when store changes
  $: {
    if (annotationLayer) {
      rebuildAnnotationLayer($annotationStore.annotations);
    }
  }

  function rebuildAnnotationLayer(annotations: Annotation[]) {
    annotationLayer.destroyChildren();
    annotationLayer.add(transformer);
    for (const ann of annotations) {
      const node = createKonvaNode(ann);
      node.on('click', () => selectAnnotation(ann.id));
      annotationLayer.add(node);
    }
    annotationLayer.batchDraw();
  }

  onDestroy(() => stage?.destroy());
</script>

<div bind:this={containerEl} class="canvas-container" />
```

### 8.3 Annotation Object Model

```typescript
// src/lib/engine/AnnotationEngine.ts
import { writable, get } from 'svelte/store';
import type { Annotation, AnnotationDocument } from '$lib/types';
import { UndoRedoStack } from './UndoRedoStack';
import { AddAnnotationCommand, DeleteAnnotationCommand, MutateAnnotationCommand } from './commands';

export class AnnotationEngine {
  private undoStack = new UndoRedoStack(100);
  document = writable<AnnotationDocument>(this.initialDoc);

  constructor(private initialDoc: AnnotationDocument) {}

  addAnnotation(ann: Annotation): void {
    const cmd = new AddAnnotationCommand(this, ann);
    this.undoStack.execute(cmd);
  }

  deleteAnnotation(id: string): void {
    const doc = get(this.document);
    const index = doc.annotations.findIndex(a => a.id === id);
    if (index === -1) return;
    const cmd = new DeleteAnnotationCommand(this, id, index);
    this.undoStack.execute(cmd);
  }

  mutateAnnotation(id: string, patch: Partial<Annotation>): void {
    const doc = get(this.document);
    const ann = doc.annotations.find(a => a.id === id);
    if (!ann) return;
    const cmd = new MutateAnnotationCommand(this, id, ann, { ...ann, ...patch });
    this.undoStack.execute(cmd);
  }

  // Direct mutations (called only by commands, not by UI)
  addAnnotationDirect(ann: Annotation): void {
    this.document.update(doc => ({
      ...doc,
      annotations: [...doc.annotations, ann],
    }));
  }

  removeAnnotationDirect(id: string): void {
    this.document.update(doc => ({
      ...doc,
      annotations: doc.annotations.filter(a => a.id !== id),
    }));
  }

  undo() { this.undoStack.undo(); }
  redo() { this.undoStack.redo(); }
  canUndo() { return this.undoStack.canUndo; }
  canRedo() { return this.undoStack.canRedo; }
}
```

### 8.4 Undo/Redo Command System

```typescript
// src/lib/engine/UndoRedoStack.ts
export interface Command {
  execute(): void;
  undo(): void;
  description: string;
}

export class UndoRedoStack {
  private undoStack: Command[] = [];
  private redoStack: Command[] = [];

  constructor(private maxDepth: number = 100) {}

  execute(cmd: Command): void {
    cmd.execute();
    this.undoStack.push(cmd);
    if (this.undoStack.length > this.maxDepth) {
      this.undoStack.shift(); // drop oldest
    }
    this.redoStack = []; // new action clears redo
  }

  undo(): void {
    const cmd = this.undoStack.pop();
    if (!cmd) return;
    cmd.undo();
    this.redoStack.push(cmd);
  }

  redo(): void {
    const cmd = this.redoStack.pop();
    if (!cmd) return;
    cmd.execute();
    this.undoStack.push(cmd);
  }

  get canUndo() { return this.undoStack.length > 0; }
  get canRedo() { return this.redoStack.length > 0; }
}
```

### 8.5 Exporting the Annotated Image

```typescript
// src/lib/export/exportImage.ts
import { invoke } from '@tauri-apps/api/core';

export async function exportToClipboard(stage: Konva.Stage): Promise<void> {
  const dataUrl = stage.toDataURL({
    pixelRatio: window.devicePixelRatio,
    mimeType: 'image/png',
  });

  // Strip the "data:image/png;base64," prefix
  const base64 = dataUrl.split(',')[1];
  await invoke('export_to_clipboard', { imageData: base64 });
}

export async function exportToFile(
  stage: Konva.Stage,
  settings: OutputSettings
): Promise<void> {
  const mimeType = settings.format === 'jpeg' ? 'image/jpeg' : 'image/png';
  const quality = settings.format === 'jpeg' ? settings.jpegQuality / 100 : undefined;

  const dataUrl = stage.toDataURL({
    pixelRatio: settings.retinaExport ? window.devicePixelRatio : 1,
    mimeType,
    quality,
  });

  const base64 = dataUrl.split(',')[1];

  // Ask Rust to open a native save dialog
  const savePath = await invoke<string | null>('open_save_dialog', {
    defaultName: expandFilenamePattern(settings.filenamePattern),
  });

  if (savePath) {
    await invoke('export_to_file', { imageData: base64, path: savePath });
  }
}
```

---

## 9. Performance Strategy

### 9.1 Capture Latency

The hotkey-to-editor target is **< 200ms**. Breakdown budget:

| Step | Target |
|------|--------|
| Hotkey detection to IPC call | < 5ms |
| macOS SCK capture | < 30ms |
| IPC transfer (PNG base64) | < 20ms |
| Tauri window creation | < 50ms |
| Konva stage init + image load | < 60ms |
| **Total** | **< 165ms** |

Key optimizations:
- Pre-create the editor Tauri window on startup and keep it hidden. On capture, show it and load the new image. Eliminates window creation latency.
- Use `SCStreamConfiguration` with `minimumFrameInterval` = 0 for single-frame capture.
- Transfer image as base64 string over IPC (no shared memory needed at this size).

### 9.2 Smooth 60fps Drawing

- **Konva batch draws**: All annotation mutations within a single event handler call `annotationLayer.batchDraw()` once, not per-annotation.
- **Preview layer separation**: The in-progress shape during drawing is on its own `previewLayer`. Only that layer is redrawn on every `mousemove`; the annotation layer with all finalized shapes is not touched.
- **Pointer event throttling**: If `mousemove` fires faster than 16ms (> 60fps), skip intermediate events using `requestAnimationFrame` gating.
- **Blur pre-computation**: Blur regions are computed once (on mouse-up) and cached as an `OffscreenCanvas`. The Konva node renders this cached bitmap — no per-frame convolution.

### 9.3 Memory Footprint

- **Base image storage**: The base image data URL is stored once in the document model. Konva's `Image` node holds a reference; no duplication.
- **Undo stack**: Capped at 100 commands. Commands store diffs (old/new state), not full document snapshots. For a document with 50 annotations, the undo stack uses ~2MB maximum.
- **Idle memory**: When the editor is not open, the hidden Tauri window uses ~15MB for WebView overhead. Total app memory at idle: ~25–30MB.

### 9.4 Large Image Handling

For scrolling captures (images > 4000px tall):
- **Tiled rendering**: The base image is split into tiles of 512×512px. Only visible tiles are composited on the Konva base layer.
- **Zoom levels**: Canvas zoom is applied via `stage.scale()`. Annotations are always in logical coordinates; Konva handles scaling.
- **Pan performance**: `stage.position()` updates are O(1) — Konva applies a CSS transform to the stage container.

### 9.5 Incremental Rendering

The annotation layer only redraws when the `annotations` array in the Svelte store changes. Svelte's reactivity system ensures the `$:` reactive block in `Canvas.svelte` only fires on actual mutations, not on every render cycle.

---

## 10. Development Roadmap

### Phase 1 — Foundation (Weeks 1–3)

**Goal**: Hotkey → region select → capture → editor window with raw image displayed.

- [ ] Tauri project scaffold (`cargo tauri init`, Svelte frontend)
- [ ] `tauri-plugin-snapink-capture`: macOS `SCScreenshotManager` implementation
- [ ] `HotkeyService`: register `⌘⇧4` globally, emit `capture:start` event
- [ ] `CaptureOverlay`: fullscreen borderless Svelte window, rubber-band selection UI
- [ ] IPC: `capture_region` command returns base64 PNG
- [ ] `EditorWindow`: opens with captured image on Konva base layer
- [ ] Basic macOS permission prompt for Screen Recording

**Deliverable**: Pressing `⌘⇧4`, drag a region, see the captured region in a blank editor window.

### Phase 2 — Core Annotation MVP (Weeks 4–7)

**Goal**: Draw rectangles, arrows, and text. Undo/redo. Copy to clipboard.

- [ ] Konva stage setup with 4-layer architecture
- [ ] `AnnotationEngine` with Svelte stores
- [ ] `UndoRedoStack` with `AddAnnotationCommand` and `MutateAnnotationCommand`
- [ ] `RectTool`: draw rectangles with Shift-constrain
- [ ] `ArrowTool`: draw arrows with angle snap
- [ ] `TextTool`: inline text edit via `Konva.Text` + custom input overlay
- [ ] `ToolController`: active tool switching, keyboard shortcuts
- [ ] Color palette bar: 8 presets + system color picker
- [ ] `ExportService`: copy annotated image to clipboard (macOS)
- [ ] Toolbar: visual states for all Phase 2 tools

**Deliverable**: Can annotate a screenshot with rectangles, arrows, and text; undo mistakes; copy result to clipboard.

### Phase 3 — Full Tool Set (Weeks 8–11)

**Goal**: Complete all annotation tools, add save-to-file, add window capture mode.

- [ ] `EllipseTool`, `LineTool`, `BrushTool` (with Catmull-Rom smoothing)
- [ ] `BlurTool`: region blur/pixelate with cached off-screen canvas
- [ ] `StepTool`: auto-incrementing numbered badges
- [ ] `SelectionManager`: click-to-select, `Konva.Transformer` resize/move
- [ ] Object context bar: color, duplicate, delete
- [ ] `ExportService`: save to file with native dialog, filename templates
- [ ] `WindowCapture`: window enumeration, hover highlight, click to capture
- [ ] Full-screen capture mode
- [ ] Settings window: General, Shortcuts, Output tabs
- [ ] Light/dark theme support via CSS variables

**Deliverable**: Feature-complete annotation editor for all P0 features in the prototype.

### Phase 4 — Advanced Features (Weeks 12–15)

**Goal**: Scrolling capture, pin image, OCR, refined UX.

- [ ] `ScrollCapture`: frame-by-frame capture, template matching stitcher
- [ ] `PinWindowManager`: always-on-top Tauri window, opacity slider, resize handle
- [ ] `OCREngine` (macOS): wrap `VNRecognizeTextRequest` via Rust FFI
- [ ] Tool options popovers (Arrow, Rectangle, Text, Blur options)
- [ ] Zoom + pan in editor (Konva stage scale + position)
- [ ] Color picker from live screen (magnifier loupe)
- [ ] Windows capture backend (`Windows.Graphics.Capture`)
- [ ] Linux capture backend (xdg-portal + PipeWire)
- [ ] Global hotkey customization UI

**Deliverable**: All P1 features complete; cross-platform capture working.

### Phase 5 — Polish and Release (Weeks 16–18)

**Goal**: Performance audit, accessibility, packaging, auto-update.

- [ ] Capture latency profiling and optimization (target < 200ms)
- [ ] Memory profiling (idle < 30MB)
- [ ] Full keyboard accessibility in editor
- [ ] macOS code signing + notarization
- [ ] Windows MSIX packaging
- [ ] Linux AppImage + Flatpak
- [ ] `tauri-plugin-updater` integration
- [ ] Crash reporting (Sentry via Tauri plugin)
- [ ] Beta testing program

---

## 11. Testing Strategy

### 11.1 Unit Tests

**Rust backend** (cargo test):
- `capture_region` mock: verify coordinate mapping and scale factor handling
- `export_to_clipboard`: verify base64 decode + clipboard write
- Settings serialization/deserialization round-trip
- Filename pattern expansion

**TypeScript frontend** (Vitest):
- `UndoRedoStack`: execute/undo/redo sequences, max depth eviction
- `AnnotationEngine`: add/remove/mutate, undo integration
- Hit testing functions: `pointNearSegment`, `pointInRect`
- Arrow angle snap: `snapAngle()`
- Step counter auto-increment: `nextStepNumber()`
- Blur computation: pixelate block average correctness

### 11.2 Component Tests

Svelte component tests via `@testing-library/svelte`:
- `ColorPalette`: swatch click updates `toolStore.color`
- `ToolBar`: active tool styling, keyboard shortcut triggering
- `ToolOptionsPopover`: slider interaction updates tool settings
- `EditorWindow`: renders correctly with empty and populated annotation store

### 11.3 Visual Regression Tests

Use **Playwright** with screenshot comparisons:
- Capture a reference screenshot of the editor with known annotations
- After each code change, re-render and compare pixel-by-pixel (tolerance: 0.1%)
- Test cases: arrow with filled head, rectangle with rounded corners, text with background, blurred region, step badge

Reference images are stored in `tests/snapshots/` and updated explicitly with `--update-snapshots`.

### 11.4 End-to-End Tests

Tauri provides a WebDriver-compatible interface. E2E tests cover the critical user path:
1. App launches → menu bar icon visible
2. Simulated `⌘⇧4` → capture overlay opens
3. Drag region → editor opens with captured image
4. Draw rectangle → annotation appears on canvas
5. `⌘Z` → annotation removed
6. `⌘C` → clipboard contains PNG (verified by reading clipboard)

### 11.5 Performance Tests

- **Capture latency**: Automated timing of hotkey → editor ready (run 100 times, assert p95 < 200ms)
- **Draw fps**: Synthetic test draws 200 annotation nodes and measures `requestAnimationFrame` callback interval (target: stable 60fps)
- **Memory**: Rust memory profiler (Heaptrack) measures RSS at idle and after 10 captures

---

## 12. Packaging and Distribution

### 12.1 Build Pipeline

```bash
# Development
npm run tauri dev

# Production builds
npm run tauri build -- --target aarch64-apple-darwin  # macOS Apple Silicon
npm run tauri build -- --target x86_64-apple-darwin   # macOS Intel
npm run tauri build -- --target universal-apple-darwin # Universal binary

npm run tauri build -- --target x86_64-pc-windows-msvc  # Windows
npm run tauri build -- --target x86_64-unknown-linux-gnu # Linux
```

The Tauri bundler produces platform-appropriate outputs:
- **macOS**: `.app` bundle + `.dmg` installer
- **Windows**: `.exe` installer (NSIS) + `.msi` (WiX)
- **Linux**: `.AppImage` + `.deb` + `.rpm`

### 12.2 macOS Distribution

**Code signing**: Developer ID Application certificate. All binaries and frameworks must be signed.

**Notarization**: Submit to Apple Notary Service via `xcrun notarytool`. Staple the notarization ticket to the `.dmg`.

**Entitlements** (required in `entitlements.plist`):
```xml
<key>com.apple.security.screen-capture</key><true/>
<key>com.apple.security.cs.allow-jit</key><true/>
```

Screen Recording permission must be requested at runtime on first launch.

**Distribution channels**:
- Direct download from website (primary)
- Homebrew Cask: `brew install --cask snapink`
- Mac App Store: Requires sandboxing adjustments; deferred to v1.1

### 12.3 Windows Distribution

**Code signing**: EV certificate required for SmartScreen bypass. Sign with `signtool.exe`.

**Distribution**:
- Direct NSIS installer (`.exe`) from website
- Winget package: `winget install snapink`
- Microsoft Store: Future consideration

### 12.4 Linux Distribution

- **AppImage**: Self-contained, runs on any Linux distribution (glibc 2.17+)
- **Flatpak**: For GNOME/KDE users; published to Flathub
- **AUR**: Maintained PKGBUILD for Arch Linux users
- **Debian/Ubuntu**: `.deb` package with appropriate depends

### 12.5 Auto-Update

`tauri-plugin-updater` fetches update manifests from a CDN endpoint. Update manifest is a signed JSON file:

```json
{
  "version": "1.2.0",
  "notes": "Bug fixes and performance improvements",
  "pub_date": "2026-04-01T00:00:00Z",
  "platforms": {
    "darwin-aarch64": { "url": "...", "signature": "..." },
    "darwin-x86_64":  { "url": "...", "signature": "..." },
    "windows-x86_64": { "url": "...", "signature": "..." },
    "linux-x86_64":   { "url": "...", "signature": "..." }
  }
}
```

Updates are downloaded in the background and applied on next launch. The user is notified via a menu bar badge.

---

## 13. Future Extensions

### 13.1 OCR on Windows and Linux

macOS uses Vision framework (built in Phase 4). Windows and Linux need an alternative:
- **Windows**: Windows.Media.Ocr (built into Windows 10+) or Tesseract FFI
- **Linux**: Tesseract OCR via `tesseract-sys` crate
- Ship as optional features (`tauri-plugin-snapink-ocr`) that are downloaded separately to keep the base binary small.

### 13.2 Cloud Sharing

A one-click upload flow:
- Configure S3-compatible endpoint (or SnapInk's own CDN) in Settings
- After annotation, click "Share" → upload PNG → copy short URL to clipboard
- End-to-end encrypted uploads (client-side encryption before upload)
- Configurable link expiry (1 hour, 24 hours, 1 week, permanent)

### 13.3 Backdrop / Beautification Tool

Add a "Backdrop" mode that wraps the screenshot in a presentation frame:
- Gradient or solid color background with configurable padding
- Drop shadow with adjustable offset, blur, opacity
- Rounded corners on the screenshot
- Window chrome overlay (macOS traffic lights, optional)
- Output optimized for blog posts and social media (1200×630 for Twitter cards)

### 13.4 Collaborative Annotations

For teams: share a session link that allows multiple users to view and annotate the same screenshot in real-time:
- WebSocket-based CRDT for annotation synchronization (Yjs)
- Presence indicators (colored cursors per user)
- Comment threads on annotations
- Export the annotated session as a single PNG or interactive HTML

### 13.5 Plugin Architecture

A lightweight plugin system for extending tools:
- Plugins are WebAssembly modules loaded at runtime
- Plugin API exposes: `onCapture(imageData)`, `registerTool(definition)`, `addMenuItem(item)`
- Example plugins: Jira upload, Linear attachment, GIMP integration, custom watermark
- Plugin discovery via a JSON registry; one-click install from the Settings UI

### 13.6 AI Annotation Assistant

Integrate with a local or remote model:
- **Auto-label**: After capture, detect UI elements (buttons, text fields) and offer auto-annotations
- **Smart blur**: Detect PII (emails, credit cards, names) and suggest blur regions
- **Caption generation**: Generate alt text or commit message from the screenshot content
- **Arrow suggestions**: Detect the most visually prominent element and suggest an arrow pointing to it
- Runs locally via an ONNX model (privacy-first), or optionally via API

---

*This document is the authoritative implementation specification for SnapInk v1.0. All architectural decisions made here should be reviewed before deviation. Questions and proposed changes should be submitted as pull requests against this document.*
