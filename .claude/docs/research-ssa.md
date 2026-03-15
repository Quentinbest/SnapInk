# Research: Screenshot Annotation Tools — Working Principles

> Comprehensive study of screenshot annotation tools: architecture, screen capture mechanisms, annotation engines, and advanced features.

---

## Table of Contents

1. [Overview of Reference Tools](#1-overview-of-reference-tools)
2. [Screen Capture Mechanisms](#2-screen-capture-mechanisms)
3. [Annotation Engine Architecture](#3-annotation-engine-architecture)
4. [Core Annotation Tools](#4-core-annotation-tools)
5. [Advanced Features](#5-advanced-features)
6. [Rendering & Performance](#6-rendering--performance)
7. [Clipboard & Export Pipeline](#7-clipboard--export-pipeline)
8. [Configuration & Extensibility](#8-configuration--extensibility)
9. [Architecture Comparison](#9-architecture-comparison)
10. [Key Takeaways for Building a Screenshot Annotation Tool](#10-key-takeaways-for-building-a-screenshot-annotation-tool)

---

## 1. Overview of Reference Tools

### 1.1 Shottr (macOS, Closed Source)

- **Platform**: macOS only (Catalina 10.15+), optimized for Apple Silicon
- **Size**: ~2.3 MB binary
- **Performance**: ~17ms capture, ~165ms to display editor
- **Tech**: Native macOS app, written in **Swift** with AppKit, CoreGraphics/Quartz, Vision framework
- **Binary**: Universal binary (Apple Silicon + Intel)
- **License**: Pay-what-you-want with free tier ("Club Shottr" ~$8–12 one-time)

**Key Features**:
- 16 annotation instruments (text, arrows, curved arrows, rectangles, ovals, freehand, highlighter, spotlight, step counter, blur/pixelate, color picker, ruler)
- Scrolling screenshot capture
- OCR text recognition & QR code reading
- Color picker with HEX, RGB, OKLCH, APCA formats
- Backdrop tool (gradient backgrounds, shadows, rounded corners)
- Before/after GIF creation
- Pin screenshots as floating windows
- Object removal via selection + delete
- QR code detection and decoding
- Pin screenshot as floating always-on-top window
- S3 upload support
- Raycast Extension / Alfred Workflow integration
- Menu bar app (no dock icon by default)
- Drag-and-drop sharing directly into other apps

### 1.2 Xnip (macOS, Closed Source)

- **Platform**: macOS only (Mac App Store)
- **Tech**: Native macOS app (Swift/Objective-C, AppKit)
- **Distribution**: Mac App Store (App ID: 1221250572)
- **License**: Freemium (free tier adds watermark; Pro unlocks scrolling capture, removes watermark)

**Key Features**:
- Rich annotation toolset: rectangle, ellipse, line, arrow, pen, highlight, text, numbered steps, blur/mosaic
- Scrolling capture for long content (source code, articles, chat histories) via **XnipHelper** component
- Window capture with shadow effects (like macOS native)
- Multi-window capture (hold Shift)
- Pin screenshots as floating windows
- Resizable capture region during annotation (unique UX feature — resize while annotating)
- Color picker for annotation colors, adjustable stroke width
- Configurable global hotkeys
- Menu bar app for quick access
- Drag-and-drop output to other apps

### 1.3 ksnip (Cross-Platform, Open Source)

- **Platform**: Linux, Windows, macOS
- **Language**: C++17
- **Framework**: Qt5 / Qt6
- **Build**: CMake
- **License**: GPL
- **Architecture**: Modular — core annotation delegated to **kImageAnnotator** library

**Key Features**:
- Full capture suite: rectangular area, full-screen, current monitor, active window, window-under-cursor
- Annotation: pen, marker, rectangles, ellipses, text, stickers, numbered markers, number arrows, blur, pixelate, duplicate tool
- Effects: drop shadow, grayscale, color inversion, borders, watermark
- Callout annotations (text box with pointer tail)
- OCR via plugin (ksnip-plugin-ocr, wraps Tesseract)
- Upload: imgur (anon/auth), FTP, custom scripts
- PDF/PS export and printing
- Tab-based multi-image editing
- Global hotkeys (Windows/X11)
- Single-instance mode
- Configurable post-capture actions/workflows (auto-copy, auto-save, auto-upload)
- Command-line interface for scripted captures
- Plugin architecture for extensibility

**Core Dependencies**:
- `kImageAnnotator` — C++ library handling all annotation rendering and interaction
- `kColorPicker` — Color selection widget
- Qt (Widgets, Network, Xml, PrintSupport, Svg, DBus)

### 1.4 Satty (Linux, Open Source)

- **Platform**: Linux/BSD (Wayland-native, wlroots compositors: Sway, Hyprland, River)
- **Language**: Rust
- **Framework**: GTK 4 + Adwaita
- **Rendering**: OpenGL (hardware-accelerated via libepoxy)
- **Build**: Cargo + Makefile
- **License**: MPL-2.0

**Key Features**:
- 11 annotation tools: pointer, crop, brush, line, arrow, rectangle, ellipse, text, numbered marker, blur, highlight
- Hardware-accelerated rendering (OpenGL)
- Fullscreen annotation mode
- Arrow angle-snapping (Shift key)
- Rectangle rounded corners (configurable 0–12+ px)
- Text: multi-line, clipboard, word navigation, IME support
- Experimental: brush smoothing, zoom, pan
- Configurable via TOML (general, keybinds, font with CJK fallback, color palette)
- Composable workflow: pipes from `grim`/`slurp`, pipes to ImageMagick
- Flatpak available

**Core Dependencies**:
- gtk4, libadwaita, libepoxy, gdk-pixbuf2, glib2, fontconfig

---

## 2. Screen Capture Mechanisms

Screen capture is the foundational layer. Each OS provides different APIs, and tools must adapt to platform-specific constraints.

### 2.1 macOS

| API | Era | Notes |
|-----|-----|-------|
| `CGWindowListCreateImage` | macOS 10.5+ | Returns composite CGImage from window list. Deprecated in macOS 15 (Sequoia). Supports window filtering, shadow inclusion/exclusion via `kCGWindowImageBoundsIgnoreFraming`. |
| `ScreenCaptureKit` (SCK) | macOS 12.3+ | Modern replacement. Uses `SCShareableContent` to enumerate windows/displays, `SCContentFilter` to select targets, `SCStreamConfiguration` for format/options. |
| `SCScreenshotManager` | macOS 14+ | Static methods for single-frame capture (returns `CGImage` or `CMSampleBuffer`). No object instantiation needed. Supports advanced filtering, pixel formats, color spaces, cursor visibility. |

**Workflow** (modern):
1. Query `SCShareableContent` for available windows/displays
2. Create `SCContentFilter` targeting desired content
3. Configure `SCStreamConfiguration` (resolution, pixel format, color space)
4. Call `SCScreenshotManager.captureImage(contentFilter:configuration:)` → `CGImage`
5. User must grant Screen Recording permission in System Settings

**Key considerations**:
- Retina/HiDPI: capture returns 2x pixel density; must handle scale factor
- Multi-monitor: each display has its own coordinate space
- Window shadows: optional inclusion via image options
- Authorization: required since macOS 10.15 for screen content access

### 2.2 Windows

| API | Era | Notes |
|-----|-----|-------|
| GDI `BitBlt` | Win32 legacy | `CreateCompatibleBitmap` + `BitBlt` from screen DC. Misses hardware-accelerated overlays (DirectX, video). CPU-intensive. |
| DXGI Desktop Duplication | Win8+ | GPU-backed. Captures full desktop including DirectX content. Provides dirty rectangles for efficient delta updates. Best performance. |
| Windows Graphics Capture | Win10 1903+ | `GraphicsCaptureSession` API. GPU-backed frames via HWND/HMON interop. Modern, supports window-specific and monitor-specific capture. |

**BitBlt Workflow**:
1. `GetDC(NULL)` → screen device context
2. `CreateCompatibleDC` + `CreateCompatibleBitmap`
3. `BitBlt(hdcMem, 0, 0, w, h, hdcScreen, 0, 0, SRCCOPY)`
4. `GetDIBits` → raw pixel data

**DXGI Duplication Workflow**:
1. `IDXGIOutput1::DuplicateOutput` → `IDXGIOutputDuplication`
2. `AcquireNextFrame` → `IDXGIResource` (GPU texture)
3. Map texture to CPU memory for processing
4. Provides cursor position/shape separately

### 2.3 Linux

| Environment | API | Notes |
|-------------|-----|-------|
| X11 | `XGetImage` / `XShmGetImage` | Direct framebuffer access. `XComposite` extension for individual window capture with transparency. No permission model. |
| Wayland | `xdg-desktop-portal` + PipeWire | Security-first. No direct framebuffer access. App requests via D-Bus (`org.freedesktop.portal.Screenshot`). Compositor streams via PipeWire. User must explicitly approve. |
| wlroots (Sway, etc.) | `grim` + `slurp` | `grim` captures via `wlr-screencopy-unstable-v1` protocol. `slurp` provides region selection UI. Output as PNG or raw PPM. |

**Wayland Portal Workflow**:
1. App calls `org.freedesktop.portal.Screenshot.Screenshot()` via D-Bus
2. Desktop portal implementation (GNOME, KDE, etc.) shows permission dialog
3. Compositor captures content → PipeWire stream
4. App receives image data from PipeWire
5. Desktop-specific portal implementations: `xdg-desktop-portal-gnome`, `xdg-desktop-portal-kde`

**Key challenge**: Wayland deliberately prevents applications from accessing screen content without explicit user consent, making traditional screenshot approaches impossible. Tools like Satty solve this by acting as a post-processor: `grim` captures, pipes to Satty for annotation.

---

## 2.5 Region Selection UI

The region selection overlay is a critical UX component. It is how users define what to capture.

**Implementation**:
1. Create a **borderless, transparent, fullscreen window** placed above all other windows (one per monitor in multi-monitor setups)
2. Optionally freeze the screen by capturing a full-screen snapshot and displaying it as the window background (prevents content from changing during selection)
3. Display crosshair cursor with pixel coordinates
4. Show magnifier/loupe near cursor for pixel-precise selection
5. On mouse-down: record start point; on drag: draw rubber-band rectangle; display width × height
6. On mouse-up: finalize selection, capture the selected region from actual screen content

**Window Detection** (for window capture mode):
- macOS: `CGWindowListCopyWindowInfo(.optionOnScreenOnly, kCGNullWindowID)` returns list of windows with bounds, owner name, window ID
- Windows: `EnumWindows` + `GetWindowRect`, or `WindowFromPoint` for hover detection
- As cursor moves, highlight the window under cursor with a colored border overlay
- Click to capture that specific window

**Keyboard modifiers during selection**:
- **Shift**: constrain to square
- **Option/Alt**: expand from center
- **Space**: move the entire selection
- **Arrow keys**: nudge selection 1px (Shift+Arrow: 10px)
- **Escape**: cancel

### 2.6 Global Hotkey System

Screenshot tools must respond to keyboard shortcuts even when not focused.

| Platform | API | Notes |
|----------|-----|-------|
| macOS | `CGEvent.tapCreate` / Carbon `RegisterEventHotKey` | Carbon API still commonly used. Requires Accessibility permission. Libraries: `HotKey`, `MASShortcut`. |
| Windows | `RegisterHotKey` (Win32) | Simple API, registers system-wide hotkey by virtual key code + modifiers. |
| Linux (X11) | `XGrabKey` | Registers key grab on root window. |
| Linux (Wayland) | D-Bus global shortcuts portal | Wayland security model prevents direct key grabs; must use compositor portal. ksnip notes global hotkeys are unsupported on Wayland. |

---

## 3. Annotation Engine Architecture

### 3.1 Core Architecture Patterns

Screenshot annotation tools typically follow a layered architecture:

```
┌─────────────────────────────────────────┐
│              UI / Toolbar               │  Tool selection, settings, shortcuts
├─────────────────────────────────────────┤
│           Annotation Engine             │  Shape creation, manipulation, rendering
├──────────────┬──────────────────────────┤
│  Tool System │   Canvas / Renderer      │  Hit testing, event dispatch, compositing
├──────────────┴──────────────────────────┤
│         Image / Document Model          │  Base image + annotation layer stack
├─────────────────────────────────────────┤
│       Capture Engine / Input            │  Screen capture, file input, clipboard
├─────────────────────────────────────────┤
│       Export / Output Pipeline          │  Clipboard, file save, upload, print
└─────────────────────────────────────────┘
```

### 3.2 Document Model

The document model separates the base screenshot from annotations:

- **Base layer**: The original captured image (immutable during annotation)
- **Annotation layer(s)**: Vector objects drawn on top of the base image
- **Composite output**: Flattened rasterization of base + annotations for export

Two main approaches:

1. **Single-layer vector overlay** (Satty, Shottr): All annotations exist in one vector layer rendered on top of the base image. Simpler model, efficient for most use cases.

2. **Multi-layer composition** (kImageAnnotator/ksnip): Annotations may be grouped or layered independently. Supports effects like drop shadows that need layer separation.

### 3.3 Object Model

Each annotation is typically represented as an object with:

```
AnnotationObject {
    type: ToolType          // arrow, rectangle, text, etc.
    geometry: Shape         // points, bounds, path data
    style: Style {
        color: Color
        strokeWidth: f32
        fill: Option<Color>
        opacity: f32
        font: Option<Font>  // for text objects
    }
    zOrder: u32             // stacking order
    selected: bool
    handles: Vec<Handle>    // resize/rotation control points
}
```

### 3.4 Tool System

Tools follow a **State Machine** or **Strategy Pattern**:

```
Tool (trait/interface)
├── handleMouseDown(point) → start shape
├── handleMouseDrag(point) → update shape
├── handleMouseUp(point)   → finalize shape
├── handleKeyDown(key)     → modifier behavior (Shift for snap, etc.)
├── render(canvas)         → preview during creation
└── cursor()               → appropriate cursor icon
```

Each tool creates annotation objects differently:
- **Rectangle/Ellipse**: mouseDown sets origin, drag sets extent, Shift constrains to square/circle
- **Arrow/Line**: mouseDown sets start, drag sets end, Shift snaps to angles (0°, 45°, 90°)
- **Brush/Freehand**: accumulates points during drag, optionally smoothed
- **Text**: click places insertion point, opens inline editor
- **Blur/Pixelate**: defines region, applies filter to underlying image content

### 3.5 Hit Testing

For selecting and manipulating existing annotations:

- **Bounding box test**: Fast AABB (axis-aligned bounding box) check first
- **Precise hit test**: Point-in-polygon for shapes, distance-to-path for lines/arrows
- **Handle hit test**: Small square regions around control points for resizing
- **Z-order traversal**: Test from top to bottom, first hit wins

kImageAnnotator uses Qt's built-in scene graph (`QGraphicsScene`/`QGraphicsView`) which provides hit testing, z-ordering, and event dispatch out of the box.

Satty, using GTK4 + OpenGL, likely implements custom hit testing against the annotation object list.

### 3.6 Undo/Redo System

Standard **Command Pattern** implementation:

```
UndoStack {
    history: Vec<Command>    // executed commands
    future: Vec<Command>     // undone commands (for redo)
}

Command (trait) {
    execute()
    undo()
}

// Example commands:
AddShapeCommand { shape }
DeleteShapeCommand { shape, previousIndex }
ModifyShapeCommand { shape, oldState, newState }
```

- **Undo** (Ctrl+Z): Pop from history, call `undo()`, push to future
- **Redo** (Ctrl+Y): Pop from future, call `execute()`, push to history
- New action clears the future stack (branching history is discarded)

---

## 4. Core Annotation Tools

### 4.1 Standard Tool Set

Every screenshot annotation tool provides a subset of these:

| Tool | Geometry | Modifiers | Notes |
|------|----------|-----------|-------|
| **Arrow** | Line + arrowhead | Shift: angle snap (45°) | Head drawn as filled triangle or custom path. Curved arrows (Shottr) use Bezier control points. |
| **Line** | Two-point segment | Shift: angle snap | Simplest drawing primitive |
| **Rectangle** | AABB rect | Shift: square constraint | Optional fill, rounded corners (Satty: 0–12px radius) |
| **Ellipse** | Center + radii | Shift: circle constraint | Satty supports center-origin drawing |
| **Text** | Position + string | — | Inline editor with font, size, color. Multi-line (Shift+Enter). IME support for CJK. |
| **Brush/Pen** | Polyline / path | — | Freehand strokes. Optional smoothing (Satty: history-based, 0–10 points). |
| **Marker/Highlighter** | Semi-transparent brush | — | Blend mode: multiply or overlay for highlight effect |
| **Numbered Marker** | Circle + auto-incrementing number | — | Used for step-by-step annotations. Counter resets per session. |
| **Blur** | Region | — | Applies Gaussian/box blur or pixelation to underlying image content |
| **Pixelate** | Region | — | Downscale then upscale with nearest-neighbor interpolation |
| **Crop** | Draggable region | — | Defines output bounds. Often the first operation. |
| **Color Picker** | Point sample | — | Reads pixel color from base image. Shottr: HEX/RGB/OKLCH/APCA output. |
| **Ruler/Measure** | Two-point distance | — | Pixel distance measurement between points |
| **Spotlight** | Region (inverted mask) | — | Dims everything outside the selected area |
| **Stickers** | Image overlay | — | Pre-defined or custom image stamps |

### 4.2 Arrow Rendering

Arrows are the most complex "simple" primitive:

```
1. Draw line segment from start to end
2. Calculate arrowhead at endpoint:
   - Direction vector = normalize(end - start)
   - Perpendicular = rotate(direction, 90°)
   - Left wing  = end - direction * headLength + perpendicular * headWidth
   - Right wing = end - direction * headLength - perpendicular * headWidth
3. Fill triangle (end, leftWing, rightWing)
```

**Curved arrows** (Shottr): Use quadratic or cubic Bezier curves. Control point(s) are draggable. The arrowhead direction follows the tangent at the endpoint.

### 4.3 Text Annotation

Text is the most complex annotation type:

- **Inline editing**: Renders a text input widget overlaying the canvas at the annotation position
- **Font handling**: System font enumeration, custom font loading (Satty uses fontconfig with CJK fallback chains)
- **Multi-line**: Line breaking, paragraph alignment
- **Selection**: Click-to-place cursor, double-click to select word, triple-click for line/all
- **Clipboard**: Cut/Copy/Paste integration
- **IME**: Input Method Editor support for CJK characters (GTK/Qt provide this natively)
- **Rendering**: Text must be rasterized at export time, matching the on-screen preview exactly

### 4.4 Blur & Pixelation Algorithms

#### Gaussian Blur
- Convolves image with a 2D Gaussian kernel: `G(x,y) = (1/2πσ²) * e^(-(x²+y²)/2σ²)`
- **Separable**: Can be applied as two 1D passes (horizontal then vertical), reducing O(n²) kernel to O(2n)
- **Approximation**: 3 passes of box blur closely approximates Gaussian blur (Central Limit Theorem)
- Typical kernel size: 5×5 to 31×31 depending on desired strength

#### Box Blur
- Each pixel = average of its neighbors in a kernel window
- Very fast with running sum / integral image technique
- Less smooth than Gaussian but adequate for privacy masking

#### Pixelation
- Divide region into blocks (e.g., 8×8 pixels)
- Replace each block with its average color (or center pixel color)
- Effectively downscale + nearest-neighbor upscale

**Implementation in annotation tools**:
1. When blur region is defined, extract that rectangle from the base image
2. Apply blur/pixelation kernel to the extracted region
3. Render the processed region as part of the annotation layer
4. Must re-compute when the region is moved or resized

---

## 5. Advanced Features

### 5.1 Scrolling Screenshot

Scrolling capture creates a single long image from content that doesn't fit on one screen.

**Capture Process**:
1. Capture initial visible frame
2. Programmatically scroll the target window by a controlled amount
3. Capture next frame
4. Repeat until end of content (detected by no new content appearing)

**Image Stitching Algorithm**:

The core challenge is finding where consecutive frames overlap:

1. **Template Matching** (most robust):
   - Take a horizontal strip from the bottom of Frame N
   - Search for this strip in Frame N+1 using normalized cross-correlation
   - The match position indicates the overlap region
   - Normalized correlation coefficient: `NCC = Σ((I - Ī)(T - T̄)) / √(Σ(I - Ī)² · Σ(T - T̄)²)`
   - Robust against brightness variations between frames

2. **Column Sampling** (faster):
   - Instead of comparing full pixel grids, sample a few vertical columns (e.g., 9)
   - Compare sampled columns between frames: O(9 × height) vs O(width × height)
   - Significant speedup, works well for text-based content

3. **Stitching**:
   - Find overlap offset → append only the non-overlapping portion of Frame N+1
   - Handle sub-pixel alignment if needed
   - Blend overlapping region to hide seams (feathered blend or hard cut)

**Challenges**:
- Floating/sticky elements (headers, toolbars) that don't scroll
- Lazy-loaded content that renders during scroll
- Animations or dynamic content between captures
- Variable scroll distances (elastic scrolling, momentum)
- Tools like Shottr and Xnip handle these edge cases through heuristics and user guidance

### 5.2 OCR (Optical Character Recognition)

Two main approaches:

**Apple Vision Framework** (macOS native):
- `VNRecognizeTextRequest` class
- Highly accurate, supports 20+ languages out of the box
- GPU-accelerated on Apple Silicon
- Used by Shottr (and likely Xnip)
- No external dependencies

**Tesseract OCR** (cross-platform, open source):
- C++ library, 100+ language support
- ksnip uses via `ksnip-plugin-ocr` plugin
- Requires trained language data files (~15MB per language)
- Lower accuracy than Apple Vision for screen content but widely available

**Workflow**:
1. User selects a region (or full image)
2. Pre-process: binarize, deskew, remove noise (optional)
3. Pass image buffer to OCR engine
4. Receive structured text result (with bounding boxes per word/line)
5. Copy text to clipboard or display in overlay

### 5.3 Color Picker

**Implementation**:
1. Capture screen content or use base image pixel data
2. On mouse move: read pixel color at cursor position from image buffer
3. Display magnified view around cursor (loupe/zoom lens)
4. On click: copy color value to clipboard

**Color format conversion** (Shottr supports all):
- **HEX**: `#RRGGBB` — direct byte-to-hex conversion
- **RGB**: `rgb(R, G, B)` — raw channel values
- **OKLCH**: Perceptually uniform color space — requires matrix transforms through OKLAB intermediate
- **APCA**: Accessible Perceptual Contrast Algorithm — contrast ratio for accessibility testing

### 5.4 Pin/Float Screenshots

- Create a borderless, always-on-top, click-through (or not) window
- Display the screenshot or cropped region in this floating window
- Implementation: platform window with `alwaysOnTop` flag + no decorations
- macOS: `NSWindow` with `.floating` level
- Linux: window manager hints (`_NET_WM_STATE_ABOVE`)

### 5.5 Backdrop / Beautification

Shottr's backdrop tool adds presentation-quality framing:

1. **Gradient background**: Generate radial/linear gradient larger than the screenshot
2. **Rounded corners**: Apply corner radius mask to screenshot before compositing
3. **Drop shadow**: Render Gaussian-blurred offset copy behind the screenshot
4. **Padding**: Add configurable margin around the screenshot

This transforms raw captures into presentation-ready images (common for blog posts, documentation, social media).

### 5.6 Pixel Measurement / Ruler

- Two-point measurement: calculate Euclidean distance in pixels
- Rectangle measurement: display width × height of selection
- Guides/gridlines for alignment
- Must account for display scale factor (Retina 2×: logical pixels vs physical pixels)

---

## 6. Rendering & Performance

### 6.1 Rendering Approaches

| Approach | Used By | Pros | Cons |
|----------|---------|------|------|
| **Software (CPU) rasterization** | ksnip (Qt QPainter) | Simple, portable, well-tested | Slower for complex scenes |
| **OpenGL hardware acceleration** | Satty | Smooth rendering, handles large images well | More complex setup, shader management |
| **Native GPU frameworks** | Shottr (likely Metal/CoreGraphics) | Best macOS performance, Retina-native | Platform-specific |
| **Scene Graph** | ksnip (QGraphicsScene) | Built-in hit testing, z-ordering, transforms | Framework dependency |

### 6.2 Canvas Rendering Pipeline

```
For each frame:
1. Clear canvas (or blit background)
2. Draw base screenshot image
3. For each annotation in z-order (bottom to top):
   a. Apply transform (position, rotation, scale)
   b. Render shape geometry (fill + stroke)
   c. If blur/pixelate: composite processed region
   d. If text: render glyphs
4. If active tool has preview: render in-progress shape
5. If selection active: render selection handles + bounding box
6. Composite UI overlays (toolbar, color picker, etc.)
```

### 6.3 HiDPI / Retina Handling

- Screen capture returns physical pixels (2× on Retina)
- Annotations must be specified in logical coordinates
- Scale factor applied at render time: `physical = logical × scaleFactor`
- Export can be at 1× or 2× depending on user preference
- Satty provides `input-scale` configuration for DPI-aware displays

### 6.4 Performance Optimizations

- **Dirty rectangle tracking**: Only re-render regions that changed
- **Texture caching**: Cache rendered annotation objects as textures (especially text)
- **Lazy blur computation**: Only compute blur when the region is finalized, not during drag
- **Throttled rendering**: Limit frame rate during drag operations (16ms frame budget for 60fps)
- **PPM format for piping** (Satty/grim): Uncompressed format avoids encode/decode overhead in capture pipeline

---

## 7. Clipboard & Export Pipeline

### 7.1 Clipboard Integration

**macOS** (`NSPasteboard`):
- Write `CGImage` as TIFF/PNG data to pasteboard
- Multiple representations: image + file URL + plain text (for OCR result)
- `NSPasteboard.general.setData(pngData, forType: .png)`

**Windows** (`Clipboard API`):
- `CF_BITMAP` or `CF_DIB` format for images
- Can also write `CF_HDROP` (file path) and `CF_UNICODETEXT`
- `OpenClipboard` → `SetClipboardData` → `CloseClipboard`

**Linux** (Wayland/X11):
- Wayland: `wl-copy` command with MIME type `image/png`
- X11: `xclip` or `xsel` with target `image/png`
- Satty makes the copy command configurable: `copy-command = "wl-copy"`

### 7.2 File Export

Common output formats:
- **PNG**: Lossless, supports transparency. Default for most tools.
- **JPEG**: Lossy, smaller files. Used when transparency not needed.
- **WebP**: Modern format, good compression. Increasingly supported.
- **PDF/PS**: ksnip supports print-quality export
- **GIF**: Shottr creates before/after animations

**Filename templating** (ksnip, Satty):
- Date/time variables: `%Y-%m-%d_%H-%M-%S` (strftime / chrono format)
- ksnip wildcards: `$Y`, `$M`, `$D`, `$T`, `$C` (counter)
- Satty: chrono format specifiers + tilde expansion

### 7.3 Upload Integration

- **Imgur**: ksnip supports anonymous and authenticated upload via API
- **FTP**: ksnip supports upload to FTP servers
- **S3**: Shottr supports third-party S3-compatible storage
- **Custom scripts**: ksnip allows user-defined upload scripts

---

## 8. Configuration & Extensibility

### 8.1 Configuration Systems

| Tool | Format | Location | Hot Reload |
|------|--------|----------|------------|
| Shottr | Preferences pane | macOS defaults | Yes |
| Xnip | Preferences pane | macOS defaults | Yes |
| ksnip | Qt Settings | Platform-specific | Requires restart |
| Satty | TOML file | `$XDG_CONFIG_DIR/satty/config.toml` | No (startup) |

**Satty TOML configuration** (most detailed among the studied tools):

```toml
[general]
fullscreen = true
early-exit = false
initial-tool = "brush"
copy-command = "wl-copy"
annotation-size-factor = 1
output-filename = "~/Pictures/satty-%Y%m%d_%H%M%S.png"
save-after-copy = false
default-hide-toolbars = false

[font]
family = "Roboto"
style = "Bold"

[color-palette]
first = "#ff0000"
custom1 = "#00ff00"

[keybinds]
# Single-key tool selection
pointer = "p"
crop = "c"
brush = "b"
line = "i"
arrow = "z"
rectangle = "r"
text = "t"
marker = "m"
blur = "u"
highlight = "g"
```

### 8.2 Keyboard Shortcuts

Common patterns across tools:
- **Tool selection**: Single letter keys (Satty) or toolbar shortcuts
- **Modifiers**: Shift (constrain aspect ratio / snap angles), Ctrl (multi-select), Alt (special behavior)
- **Actions**: Ctrl+Z (undo), Ctrl+Y/Ctrl+Shift+Z (redo), Ctrl+C (copy), Ctrl+S (save)
- **Navigation**: Arrow keys (nudge 1px), Shift+Arrow (nudge 10px), Scroll/Pinch (zoom)
- **Capture**: Global hotkeys (ksnip on Windows/X11)

### 8.3 Plugin Architecture (ksnip)

ksnip supports plugins for extending functionality:

```
Plugin Detection Paths:
├── Windows: ./plugins/ (next to executable)
├── Linux:   /usr/local/lib/, /usr/lib/, /usr/lib64/
└── Custom:  Settings > Plugins > custom path

Plugin Interface:
├── Discovered automatically at startup
├── OCR Plugin: wraps Tesseract OCR engine
└── Settings accessible via Options > Settings > Plugins
```

---

## 9. Architecture Comparison

### 9.1 Feature Matrix

| Feature | Shottr | Xnip | ksnip | Satty |
|---------|--------|------|-------|-------|
| **Platform** | macOS | macOS | Cross-platform | Linux (Wayland) |
| **Language** | Swift/ObjC (likely) | Swift/ObjC (likely) | C++17 | Rust |
| **UI Framework** | AppKit/SwiftUI | AppKit | Qt5/Qt6 | GTK4 + Adwaita |
| **Rendering** | CoreGraphics/Metal | CoreGraphics | QPainter/QGraphicsScene | OpenGL |
| **Capture** | ScreenCaptureKit | ScreenCaptureKit | Platform-specific | External (grim/slurp) |
| **Scrolling capture** | Yes | Yes | No | No |
| **OCR** | Yes (Vision) | No | Plugin (Tesseract) | No |
| **Color picker** | Yes (multi-format) | No | No | No |
| **Blur/Pixelate** | Yes | Yes | Yes | Yes |
| **Numbered markers** | Yes (step counter) | No | No | Yes |
| **Pin/Float** | Yes | Yes | Yes | No |
| **Upload** | S3 | No | Imgur/FTP/Custom | No |
| **Backdrop** | Yes | No | No | No |
| **Open source** | No | No | Yes (GPL) | Yes (MPL-2.0) |

### 9.2 Architecture Patterns

**Monolithic Native** (Shottr, Xnip):
- Capture + annotate + export in single native app
- Tight OS integration, best performance
- Platform-locked

**Modular Library** (ksnip):
- Capture engine in main app
- Annotation delegated to `kImageAnnotator` library
- Color picker in separate `kColorPicker` library
- Extensible via plugin interface
- Cross-platform but complex dependency management

**Unix Pipeline** (Satty):
- Capture handled by external tools (`grim` + `slurp`)
- Satty receives image via stdin/file and provides annotation
- Export via stdout or file, clipboard via external command (`wl-copy`)
- Follows Unix philosophy: each tool does one thing well
- Most composable, least self-contained

---

## 10. Key Takeaways for Building a Screenshot Annotation Tool

### 10.1 Minimum Viable Feature Set

1. **Screen capture** — area selection, full screen, window
2. **Basic shapes** — rectangle, ellipse, line, arrow
3. **Text annotation** — inline editor with font/size/color
4. **Freehand brush** — with configurable width/color
5. **Blur/Pixelate** — region-based privacy masking
6. **Undo/Redo** — command pattern with history stack
7. **Clipboard** — copy annotated result to clipboard
8. **File save** — PNG export with configurable path

### 10.2 Architecture Recommendations

1. **Separate capture from annotation**: The capture engine and annotation engine should be independent modules. This enables pipeline composition (like Satty) and makes each component testable.

2. **Vector annotation model**: Store annotations as vector objects (not burned into raster). This enables non-destructive editing, undo/redo, individual object selection, and resolution-independent rendering.

3. **Hardware-accelerated rendering**: For smooth interaction with large screenshots (especially Retina/4K), GPU rendering (OpenGL, Metal, Vulkan) provides dramatically better performance than CPU rasterization.

4. **Flatten on export only**: Keep the vector annotation layer separate from the base image during editing. Rasterize (flatten) only when exporting to clipboard or file.

5. **HiDPI-first design**: Handle display scale factors from the beginning. All coordinates should be in logical pixels, with scale factor applied at render time.

### 10.3 Technical Challenges

| Challenge | Solution |
|-----------|----------|
| Retina/HiDPI coordinates | Logical coordinate system + scale factor at render |
| Multi-monitor capture | Per-display coordinate spaces, composite or per-monitor capture |
| Wayland screen access | Portal API + PipeWire, or external capture tool pipeline |
| Smooth freehand drawing | Point smoothing (Catmull-Rom spline, moving average) |
| Large image performance | Tiled rendering, dirty rect tracking, texture caching |
| Text rendering quality | Subpixel antialiasing, proper font metrics, harfbuzz for complex scripts |
| Arrow head at any angle | Vector rotation math, tangent calculation for curved arrows |
| Blur performance | Separable kernel, integral image for box blur, compute on GPU |
| Scrolling capture | Template matching for overlap detection, floating element handling |

### 10.4 Other Notable Tools Worth Studying

| Tool | Platform | Tech Stack | Noteworthy For |
|------|----------|------------|----------------|
| **Flameshot** | Linux/Win/macOS | C++/Qt | Full-featured open source, good codebase to study |
| **ShareX** | Windows | C#/.NET | Extremely feature-rich, workflow automation |
| **Greenshot** | Windows | C#/.NET | Clean plugin architecture |
| **CleanShot X** | macOS | Swift/AppKit | Gold standard UX for macOS screenshot tools |
| **Spectacle** | Linux (KDE) | C++/Qt | Proper Wayland integration via KWin |
| **NormCap** | Cross-platform | Python | OCR-focused screenshot tool (captures text, not images) |

### 10.5 Technology Stack Options

**macOS Native**: Swift + AppKit/SwiftUI + ScreenCaptureKit + CoreGraphics/Metal
- Best performance and OS integration
- Access to Vision framework for OCR
- Smallest binary size

**Cross-Platform (Qt)**: C++ + Qt6 + platform capture APIs
- Proven approach (ksnip)
- QGraphicsScene provides scene graph with hit testing
- Large dependency (Qt framework)

**Cross-Platform (Rust + GTK4)**: Rust + GTK4 + OpenGL
- Modern approach (Satty)
- Memory safety, good performance
- GTK4 provides GPU-accelerated rendering

**Cross-Platform (Electron/Tauri)**: TypeScript/Rust + Canvas/WebGL
- Tauri: Rust backend + web frontend, small binary
- HTML Canvas for annotation rendering
- Fabric.js or Konva.js for canvas object model
- Tradeoff: larger bundle, potential performance overhead

---

## References

### Tools Studied
- [Shottr](https://shottr.cc/) — macOS screenshot annotation app
- [Xnip](https://xnipapp.com/) — macOS screenshot tool with scrolling capture
- [ksnip](https://github.com/ksnip/ksnip) — Cross-platform Qt-based screenshot tool
- [kImageAnnotator](https://github.com/ksnip/kImageAnnotator) — C++ annotation library used by ksnip
- [Satty](https://github.com/Satty-org/Satty) — Rust/GTK4 screenshot annotation tool for Wayland

### Platform APIs
- [ScreenCaptureKit (Apple)](https://developer.apple.com/documentation/screencapturekit/) — Modern macOS screen capture framework
- [CGWindowListCreateImage (Apple)](https://developer.apple.com/documentation/coregraphics/1454852-cgwindowlistcreateimage) — Legacy macOS capture API (deprecated)
- [Windows GDI Capture](https://learn.microsoft.com/en-us/windows/win32/gdi/capturing-an-image) — Win32 BitBlt-based capture
- [DXGI Desktop Duplication](https://www.codeproject.com/Tips/1116253/Desktop-Screen-Capture-on-Windows-via-Windows-Desk) — GPU-backed Windows capture
- [xdg-desktop-portal](https://adangel.org/2023/05/31/screencapture-api-wayland/) — Wayland screenshot portal API

### Algorithms & Techniques
- [Scrolling Screenshot Stitching Principles (Deepin)](https://medium.com/@deepinlinux/technical-sharing-screen-capture-principles-of-long-screenshots-418e59d81d3c)
- [Gaussian Blur (Wikipedia)](https://en.wikipedia.org/wiki/Gaussian_blur)
- [Box Blur Algorithm](https://www.geeksforgeeks.org/box-blur-algorithm-with-python-implementation/)
- [Apple Vision Text Recognition (WWDC19)](https://developer.apple.com/videos/play/wwdc2019/234/)
- [Canvas Layer Architecture with Konva](https://medium.com/htc-research-engineering-blog/konva-use-konva-to-create-annotation-tool-34409bfa822b)
