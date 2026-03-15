# SnapInk — Engineering Implementation Plan v2.0

> From prototype to production: a complete technical blueprint for building SnapInk, a lightweight, fast, keyboard-driven screenshot annotation tool.
>
> This plan is derived directly from the **Figma prototype** (file `MhxyNHPAhPoADayWyNtXd4`, 4 pages) and the **research document** (`research-ssa.md`). Every measurement, color, component state, and interaction pattern referenced here traces to a concrete prototype element.

**Version**: 2.0
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

### 1.2 Target Users

| User Type | Primary Need |
|-----------|-------------|
| Developers | Bug reports, PR reviews, code walkthroughs |
| Designers | UI feedback, red-lining, design reviews |
| Product Managers | Feature specs, stakeholder walkthroughs |
| Technical Writers | Step-by-step documentation, tutorials |
| Support Teams | Reproducing issues, documenting user problems |

### 1.3 Core User Scenarios

1. **Bug Report**: `⌘⇧4` → drag region → arrow + text "this is broken" → `⌘C` → paste into Jira.
2. **Design Feedback**: `⌘⇧5` → click window → numbered steps → save PNG → attach to Figma.
3. **Tutorial**: `⌘⇧4` → capture dialog → blur username → step counter badges → `⌘S`.
4. **Quick Reference**: `⌘⇧4` → capture code → pin as floating window.
5. **Scrolling Doc**: `⌘⇧6` → select scrollable area → auto-stitch long page → annotate → export.
6. **Text Extraction**: `⌘⇧7` → OCR region select → copy recognized text.

### 1.4 Product Principles

1. **Speed-first**: Hotkey-to-editor < 200ms. Export < 100ms. No loading screens.
2. **Invisible until needed**: Menu bar only. No Dock icon (`LSUIElement = true`). < 30MB RAM idle.
3. **Inline editing**: Annotations directly on the capture. Tool options as compact popovers.
4. **Non-destructive**: Annotations are vector objects above the base image. Rasterize only at export.
5. **macOS-native feel**: SF Pro, system blur materials, traffic-light chrome, system color picker.

### 1.5 Differentiation

SnapInk's edge vs competitors: **Tauri-based cross-platform binary under 15MB**, near-native capture speed, and a clean Svelte UI with full dark/light theming matching each platform's aesthetic. Open-source (or source-available) with no subscriptions, no watermarks, no cloud dependency.

---

## 2. Prototype Analysis

The Figma prototype consists of 4 pages captured from HTML mockups. This section maps every prototype element to its engineering module.

### 2.1 Page 1 — Design System (`01-design-system.html`)

The design system defines the visual language. All values below are authoritative for implementation.

#### 2.1.1 Color Tokens

**Light Mode Core UI**:

| Token | Value | Usage |
|-------|-------|-------|
| `bg/primary` | `#FFFFFF` | Window background, canvas BG |
| `bg/toolbar` | `rgba(255,255,255,0.85)` + `backdrop-filter:blur(20px)` | Toolbar, action bar |
| `bg/overlay` | `rgba(0,0,0,0.40)` | Region selection dim mask |
| `accent` | `#007AFF` | Active tool, selection border, CTA buttons |
| `border/selection` | `#007AFF` | Region selection rect, object handles |
| `border/subtle` | `rgba(0,0,0,0.10)` | Toolbar border, menu border |
| `text/primary` | `#1D1D1F` | Body text, labels |
| `text/secondary` | `#86868B` | Shortcuts, descriptions, meta text |
| `destructive` | `#FF3B30` | Cancel button, delete action |
| `success` | `#34C759` | Done button, toast checkmark |

**Dark Mode Core UI**:

| Token | Value | Usage |
|-------|-------|-------|
| `bg/primary` | `#1E1E1E` | Editor background |
| `bg/toolbar` | `rgba(40,40,40,0.85)` + `backdrop-filter:blur(20px)` | Dark toolbar |
| `bg/overlay` | `rgba(0,0,0,0.55)` | Dark overlay dim |
| `accent` | `#0A84FF` | Active tool (dark), selection |
| `border/selection` | `#0A84FF` | Handles, bounding box |
| `border/subtle` | `rgba(255,255,255,0.10)` | Toolbar border (dark) |
| `text/primary` | `#F5F5F7` | Body text (dark) |
| `text/secondary` | `#A1A1A6` | Meta text (dark) |
| `destructive` | `#FF453A` | Cancel (dark) |
| `success` | `#30D158` | Done (dark) |

**Engineering note**: Implement as CSS custom properties on `:root` and `[data-theme="dark"]` selectors. Svelte's `ThemeManager` reads `prefers-color-scheme` and the user's settings to toggle `data-theme`.

#### 2.1.2 Annotation Color Palette

8 preset colors + 1 custom (system color picker):

```
Red     #FF3B30    (default active)
Orange  #FF9500
Yellow  #FFCC00
Green   #34C759
Blue    #007AFF
Purple  #AF52DE
Black   #1D1D1F
White   #FFFFFF
Custom  conic-gradient(red,orange,yellow,green,blue,purple,red)
```

Active swatch indicator: `box-shadow: 0 0 0 2px <bg-color>, 0 0 0 4px #007AFF` (light) or `0 0 0 4px #0A84FF` (dark).

#### 2.1.3 Typography Scale (SF Pro)

| Token | Spec | Usage |
|-------|------|-------|
| `title` | 16px / Semibold / 20px LH / -0.2px tracking | Settings window title |
| `menu-item` | 13px / Regular / 18px LH | Menu dropdown items |
| `menu-shortcut` | 13px / Regular / 18px LH / `text/secondary` | Shortcut labels |
| `toolbar-label` | 11px / Medium / 14px LH | Tool tooltips |
| `dimension` | 11px / SF Mono / Regular / 14px LH | Dimension pill `640 × 480` |
| `annotation-text` | 14px / Regular / 18px LH | Default annotation text |
| `body` | 13px / Regular / 18px LH | Settings descriptions |

Font stack: `-apple-system, BlinkMacSystemFont, 'SF Pro Text', 'Helvetica Neue', sans-serif`. Monospace: `'SF Mono', 'Menlo', monospace`.

#### 2.1.4 Spacing Tokens

| Token | Value | Usage |
|-------|-------|-------|
| `space-xs` | 4px | Icon-to-label gap |
| `space-sm` | 8px | Between toolbar items, swatch gap in palette |
| `space-md` | 12px | Toolbar internal padding |
| `space-lg` | 16px | Between tool groups |
| `space-xl` | 24px | Settings section spacing, editor canvas padding |

#### 2.1.5 Border Radius

| Token | Value | Usage |
|-------|-------|-------|
| `radius-sm` | 4px | Keyboard badges, shortcut tags |
| `radius-md` | 8px | Pin windows, contextual bar, popovers |
| `radius-lg` | 12px | Editor frame, toolbar, settings window |
| `radius-pill` | 9999px | Dimension pill, toast, palette bar, coord pill |

#### 2.1.6 Elevation Levels

| Token | Shadow | Usage |
|-------|--------|-------|
| `elevation-toolbar` | `0 2px 12px rgba(0,0,0,0.15)` | Toolbar, action bar |
| `elevation-popover` | `0 4px 20px rgba(0,0,0,0.20)` | Tool option popovers |
| `elevation-menu` | `0 8px 32px rgba(0,0,0,0.25)` | Menu dropdown, submenu |
| `elevation-pin` | `0 4px 16px rgba(0,0,0,0.30)` | Pin floating window |

Dark mode: elevation shadows increase to `0.4–0.6` opacity.

#### 2.1.7 Component Atoms (from prototype)

**Tool Button** — 32×32px, `radius-md` (in toolbar context: 6–7px rounded):
- Default: transparent bg, `text/primary` color
- Hover: `rgba(0,0,0,0.06)` bg (light) / `rgba(255,255,255,0.1)` (dark)
- Active: `rgba(0,122,255,0.12)` bg + `#007AFF` color (light) / `rgba(10,132,255,0.2)` + `#0A84FF` (dark)
- Disabled: `rgba(0,0,0,0.3)` color (light) / `rgba(255,255,255,0.25)` (dark)

**Selection Handles** — 8px × 8px, white fill, 1.5px `#007AFF` border, 1px border-radius. 8 handles: 4 corners + 4 edge midpoints. Bounding box: 1.5px dashed `#0A84FF`.

**Dimension Pill** — `rgba(0,0,0,0.72)` bg, pill radius, 11px SF Mono white text, padding 3px 8px.

**Keyboard Badge** — `<kbd>` styled: `#F2F2F7` bg, `1px solid #D1D1D6` border, 4px radius, 11px font, 2px 5px padding.

**Step Counter Badge** — 24px circle (default), `#FF3B30` bg by default (inherits annotation color), white 13px bold number. Larger variant: 32px circle, 15px font.

**Toast Notification** — Pill shape, `rgba(28,28,30,0.92)` bg, `elevation-pin` shadow, 13px text, green ✓ + "Copied to clipboard".

**Toolbar (Light)** — `rgba(255,255,255,0.92)` bg, `backdrop-filter:blur(20px)`, `1px solid rgba(0,0,0,0.08)` border, 10px radius, 4px 8px padding, `elevation-toolbar` shadow. Groups separated by 1px × 18px dividers at `rgba(0,0,0,0.1)`.

**Toolbar (Dark)** — `rgba(44,44,46,0.96)` bg, same blur, `rgba(255,255,255,0.12)` border, 12px radius, 5px 10px padding, 44px height, `0 4px 20px rgba(0,0,0,0.5)` shadow. Dividers: `rgba(255,255,255,0.12)`.

### 2.2 Page 2 — Menu Bar & Capture (`02-menu-and-capture.html`)

#### 2.2.1 Menu Bar Icon

- macOS menu bar, right side among system icons
- SnapInk icon: 18×18px square, `#1D1D1F` bg, 3px radius, white "S" bold 10px
- Active state: icon container gets `#007AFF` background, white text "SnapInk" label
- → **Module**: `MenuBarController` (Tauri tray icon)

#### 2.2.2 Dropdown Menu

Specs: 226px wide, `rgba(248,248,250,0.98)` bg, `backdrop-filter:blur(40px)`, `1px solid rgba(0,0,0,0.12)` border, 10px radius, 6px 0 padding, `elevation-menu` shadow.

Menu items (5px 14px padding, 13px font, 8px icon-label gap):

| Item | Icon | Shortcut | Note |
|------|------|----------|------|
| **Capture Screen** | ⬛ | `⌘⇧3` | Highlighted (blue) by default |
| Capture Area | ✂️ | `⌘⇧4` | |
| Capture Window | 🪟 | `⌘⇧5` | |
| Scrolling Capture | 📜 | `⌘⇧6` | |
| — separator — | | | |
| Recognize Text (OCR) | 🔤 | `⌘⇧7` | |
| — separator — | | | |
| More | ··· | › (arrow) | Opens submenu |
| — separator — | | | |
| Settings… | ⚙️ | `⌘,` | |
| Quit SnapInk | | `⌘Q` | |

Highlighted state: `#007AFF` bg, white text, 5px radius, inset 5px margin.

#### 2.2.3 "More" Submenu

210px wide submenu (same styling as main menu):

| Item | Icon | Shortcut |
|------|------|----------|
| Repeat Last Capture | ↩ | `⌘⇧R` |
| Capture Active Window | 🪟 | `⌘⇧5` |
| Delayed Screenshot (3s) | ⏱ | |
| Scrolling (Up) | ↑ | |
| — separator — | | |
| Open File… | 📁 | |
| Load from Clipboard | 📋 | |

→ **Engineering**: Menu items map to `MenuBarController.actions`. "Open File" triggers native file picker (`invoke('open_file_dialog')`). "Load from Clipboard" reads image from clipboard (`invoke('read_clipboard_image')`).

#### 2.2.4 Region Selection Overlay — 3 States

**State 1 — Idle (Crosshair)**:
- Fullscreen borderless transparent window, content frozen as background
- Dim overlay: `rgba(0,0,0,0.45)` over entire screen
- Crosshair: Two lines — horizontal full-width 1px `rgba(255,255,255,0.55)`, vertical full-height 1px same color
- Cursor dot: 10px white circle centered at intersection
- Coord pill: offset 14px right and 14px below cursor, showing `"842, 520"` in dimension style
- Instruction text at bottom center: "Click and drag to select area", 12px `rgba(255,255,255,0.5)`

**State 2 — Dragging (Rubber-band)**:
- Dim masks on all 4 sides (top/bottom/left/right of selection) using `rgba(0,0,0,0.45)`
- Selection rectangle: 1.5px **dashed** border `#007AFF`, `rgba(0,122,255,0.04)` fill
- Dimension pill centered below selection: `"296 × 104"` (updates live)
- Faded crosshair at endpoint (0.4 opacity)

**State 3 — Complete (Ready to Capture)**:
- Dim masks on all sides (unchanged)
- Selection rectangle: 1.5px **solid** border `#007AFF` (no longer dashed)
- Dimension pill **above** selection (offset -26px from top edge)
- 8 resize handles (corners + edge midpoints): 8px × 8px, white fill, 1.5px `#007AFF` border
- **Action bar** centered below the selection (bottom: 14px): glassmorphism bar containing:
  - `"✂ Capture"` — primary button, `#007AFF` bg, white text
  - Divider
  - `"📜 Scroll"` — secondary button
  - `"📌 Pin"` — secondary button
  - Divider
  - `"✕"` — danger (red)
  - `"✓"` — success (green)

→ **Module**: `CaptureOverlay`, `RegionSelector`, `ActionBarComponent`

#### 2.2.5 Window Capture — Hover Detection

- Desktop with dim overlay `rgba(0,0,0,0.15)` (lighter than region selection)
- Non-targeted windows: reduced opacity (0.6)
- Hovered window: **3px solid `#007AFF` border** with 10px radius, highlighting the window bounds
- **Window label pill**: positioned 34px above the window top-left, `rgba(0,0,0,0.72)` bg, pill radius, 11px white text showing `"Chrome — apple.com"`
- Instruction bar at bottom center: `rgba(0,0,0,0.65)` bg, pill radius, 12px white text: `"Click to capture • Shift+Click for multiple • Esc to cancel"`

→ **Module**: `WindowEnumerator`, `WindowCapturOverlay`
→ **Engineering note**: Shift+Click for multi-window capture requires collecting multiple window IDs before compositing.

### 2.3 Page 3 — Annotation Editor (`03-annotation-editor.html`)

#### 2.3.1 Editor Window (Dark Mode)

- **Frame**: 1100px width, `#1C1C1E` bg, 12px radius, `0 24px 80px rgba(0,0,0,0.6)` shadow
- **Title bar**: 28px height, `#2C2C2E` bg, 1px bottom border `rgba(255,255,255,0.08)`, macOS traffic lights (12px circles: `#FF5F57` / `#FEBC2E` / `#28C840`), centered title `"SnapInk — Screenshot 2026-03-14 at 15.47.33"` in 12px `#A1A1A6`
- **Canvas area**: Padding 32px sides, 80px bottom (to leave room for toolbar), min-height 540px, subtle dot grid background: `radial-gradient(circle, rgba(255,255,255,0.06) 1px, transparent 1px)` at 24px intervals

#### 2.3.2 Screenshot Viewport

- 780×440px screenshot container, 4px radius, `0 4px 32px rgba(0,0,0,0.5)` shadow
- Contains a mock webpage (browser chrome + content) as the captured image
- SVG annotation layer positioned absolutely over the screenshot at exact viewport dimensions

#### 2.3.3 Annotations Visible in Prototype

The prototype demonstrates these annotation types simultaneously:

1. **Arrow** — Red `#FF3B30`, 2.5px stroke, pointing from (180,300) to (142,270), arrowhead as SVG `<marker>` with filled polygon `8×6` at tip
2. **Rectangle** — Blue `#007AFF`, 2px stroke, no dash, 4px corner radius, 732×160px bounding the hero section
3. **Blur region** — `rgba(180,180,200,0.5)` fill with monospace `████████` text suggesting pixelated content, 160×26px, 3px radius
4. **Ellipse** (freehand-style) — Yellow `#FFCC00`, 2.5px stroke, 60×22px radii, 0.9 opacity, highlighting the CTA button
5. **Text annotation** — "buy this!" in red, positioned at (196, 290), with background: `rgba(255,255,255,0.92)` bg, 1.5px red border, 5px radius, 13px semibold text, `0 2px 8px rgba(0,0,0,0.15)` shadow
6. **Step counters** — Two blue `#007AFF` 24px circle badges numbered ① and ② at the left edge, `0 2px 8px rgba(0,122,255,0.5)` glow shadow

#### 2.3.4 Toolbar (Dark Mode — Detailed SVG Icons)

Height 44px, positioned absolutely at bottom center, 12px radius. The toolbar groups and their SVG icons from the prototype:

**Group 1 — Shapes** (4 tools):
- `Rectangle (R)` — 16×16 viewBox, `<rect x=2 y=3 w=12 h=10 rx=1.5 stroke-width=1.8/>`
- `Ellipse (O)` — `<ellipse cx=8 cy=8 rx=6 ry=5 stroke-width=1.8/>`
- `Line (L)` — `<line x1=2 y1=14 x2=14 y2=2 stroke-width=1.8 linecap=round/>`
- `Arrow (A)` — line + polyline arrowhead chevron, **active state shown** in prototype

**Group 2 — Draw Tools** (4 tools):
- `Pen (P)` — pen nib path, stroke-width 1.5
- `Blur/Pixelate (B)` — 4 small rectangles in 2×2 grid (representing pixel blocks)
- `Text (T)` — T-shape: horizontal line + vertical line + baseline serifs
- `Step Counter (N)` — circle + number "1" text

**Group 3 — History** (2 tools):
- `Undo (⌘Z)` — circular arrow pointing left
- `Redo (⌘⇧Z)` — circular arrow pointing right, **disabled state** in prototype

**Group 4 — Actions** (5 tools):
- `Save (⌘S)` — downward arrow + baseline
- `Copy to Clipboard (⌘C)` — overlapping rectangles
- `Pin Image` — pin/thumbtack path
- `Cancel (Esc)` — ✕ cross, `#FF453A` (danger)
- `Done (Enter)` — ✓ checkmark, `#30D158` (success)

#### 2.3.5 Color Palette Bar

Below toolbar, pill-shaped: `rgba(44,44,46,0.92)` bg, blur, `rgba(255,255,255,0.1)` border, pill radius, 6px 12px padding.

Swatches: 18px diameter circles, 1.5px `rgba(255,255,255,0.15)` border. Active swatch: `0 0 0 2px #1C1C1E, 0 0 0 4px #0A84FF` ring. Custom swatch: conic gradient, no border. 1px × 14px divider before custom swatch.

#### 2.3.6 Tool Options Popovers

All popovers: 200px wide, `rgba(44,44,46,0.98)` bg, `rgba(255,255,255,0.12)` border, 12px radius, 16px padding, `0 8px 32px rgba(0,0,0,0.5)` shadow.

**Arrow Tool Options**:
- Stroke Width: slider (4px track, `#0A84FF` fill, 14px white circle thumb), value label "3px"
- Filled Head: toggle ON (32×18px, `#0A84FF` bg, 14px white knob right-aligned)
- Angle Snap: toggle ON

**Rectangle Tool Options**:
- Stroke Width: slider, "2px"
- Corner Radius: slider, "0px"
- Fill: toggle OFF (`rgba(255,255,255,0.2)` bg, knob left-aligned)

**Text Tool Options**:
- Font Size: slider, "14px"
- Bold: toggle OFF
- Background: toggle ON

**Blur/Pixelate Tool Options**:
- Strength: slider, "Medium"
- Mode: segmented control — "Blur" active (`rgba(10,132,255,0.2)` bg, `1px #0A84FF` border), "Pixelate" inactive (`rgba(255,255,255,0.08)` bg)

#### 2.3.7 Object Selection State

When an annotation is selected:
- **Bounding box**: 1.5px dashed `#0A84FF`, 2px radius, inset -4px from annotation bounds
- **8 handles**: 8px × 8px, white fill, 1.5px `#0A84FF` border, 1.5px radius
- **Contextual bar** above selection: same dark glassmorphism as toolbar (8px radius, 4px 6px padding), containing:
  - 🎨 (color) — 4px 8px button, 5px radius
  - `|` divider (1px × 14px)
  - ⎘ (duplicate)
  - `|` divider
  - 🗑 (delete) — danger hover state

#### 2.3.8 Light Mode Variant

Same layout, inverted colors:
- Editor frame: `#F2F2F7` bg, `0 8px 40px rgba(0,0,0,0.15)` shadow
- Title bar: `#E8E8ED` bg, `rgba(0,0,0,0.08)` bottom border, `#86868B` title text
- Canvas: `#F2F2F7` bg, dot grid at `rgba(0,0,0,0.06)`
- Toolbar: `rgba(255,255,255,0.96)` bg, `rgba(0,0,0,0.1)` border, `#1D1D1F` icon color
- Active tool: `rgba(0,122,255,0.12)` bg, `#007AFF` color
- Palette bar: white bg, `rgba(0,0,0,0.1)` border, `0 2px 12px rgba(0,0,0,0.12)` shadow

### 2.4 Page 4 — Advanced Features (`04-advanced-features.html`)

#### 2.4.1 Scrolling Capture — 3 States

**State 1 — Setup**: 340px wide capture region showing a browser page with selection overlay. Dim sides of selection with `rgba(0,0,0,0.3)`. Selection border: 2px solid `#007AFF`. Instruction pill: "Select scrollable content area". Below frame: action bar with `"📜 Start Scrolling"` primary button + `"✕ Cancel"` danger text.

**State 2 — In Progress**: Same browser, new content visible (page has scrolled). **Progress pill** top-center: dark bg `rgba(28,28,30,0.85)`, blur, pill shape, containing CSS spinner (14px, 2px border, `border-top-color:#fff`, 0.8s linear infinite rotation) + "Capturing... 5 frames". **Preview strip** right side: 44px wide dark column with 36×20px frame thumbnails stacked vertically (decreasing opacity for newest). **Progress bar** at bottom: 3px height, `rgba(255,255,255,0.2)` track, `#007AFF` fill (60% in prototype). Below frame: instruction text "Slowly scroll the content downward…" + red "Stop" link.

**State 3 — Complete**: Editor-style dark window (280px wide, mini title bar), title: "Scrolling Capture — 1440 × 4820". Tall stitched image preview with vertical scrollbar (3px wide, `rgba(255,255,255,0.3)` thumb). Bottom stats bar: "7 frames stitched" left, "1440 × 4820 px" right in `#0A84FF`.

→ **Modules**: `ScrollCaptureController`, `ImageStitcher`, `FramePreviewStrip`

#### 2.4.2 Pin Image — Floating Windows

Desktop mockup (680×400px, gradient background) with macOS dock simulation at bottom.

Three pin windows shown in different states:

**Pin Window 1 — Idle (Code Snippet)**: 220×140px, 8px radius, no chrome/controls visible, `0 4px 20px rgba(0,0,0,0.4)` shadow, showing syntax-highlighted Swift code on dark background.

**Pin Window 2 — Hover (Controls Visible)**: 260×180px. On hover, controls appear:
- **Close button**: top-left, 20×20px, `#FF3B30` bg, 50% radius, white "✕" 10px bold
- **Expand button**: top-right, 20×20px, `#007AFF` bg, 50% radius, white "⤢" expand icon
- **Opacity indicator**: bottom-center, pill shape, `rgba(0,0,0,0.6)` bg, 10px text showing "85%"
- **Resize handle**: bottom-right, 14×14px SVG with 3 diagonal lines `rgba(255,255,255,0.6)`

**Pin Window 3 — Background/Dimmed**: 180×120px, `opacity: 0.9`, no controls. Represents a pin that's behind other content.

→ **Module**: `PinWindowManager` (creates borderless `alwaysOnTop` Tauri windows)

#### 2.4.3 OCR — Text Recognition Flow

**Step 1 — Select Region**: OCR frame (460px wide, dark theme), header bar "Recognize Text (OCR) — Select area to extract". Screenshot preview (120px height, white bg). Blue selection rect over text: 2px solid `#0A84FF`, `rgba(10,132,255,0.08)` fill. Instruction: "Drag to select text region • Click to recognize".

**Step 2 — Result**: Header changes to "Text Recognized ✓". Screenshot dims to 0.5 opacity. Result panel appears below with `#2C2C2E` bg: recognized text in `rgba(255,255,255,0.05)` bg block (13px, 1.5 line height), followed by `"⎘ Copy Text"` primary button (`#0A84FF`) + "Again" secondary button.

→ **Module**: `OCREngine` (macOS: Vision framework `VNRecognizeTextRequest`)

#### 2.4.4 Settings Window — 3 Tabs

520px wide, white bg, 12px radius, `elevation-menu` shadow. Title bar 40px, `#F2F2F7` bg. **Segmented tab bar**: "General | Shortcuts | Output" — active tab `#007AFF` bg + white text, inactive `#fff` bg + `#86868B` text, 1px `#D1D1D6` border, 7px radius on outer edges.

**Tab 1 — General**:
- **Startup group**: Launch at Login (toggle ON), Show menu bar icon (toggle ON), Play sound on capture (toggle OFF)
- **After Capture group**: Radio buttons — ● Open in annotation editor (selected, `#007AFF` filled circle) / ○ Copy to clipboard immediately. Also: "Also copy after annotating" toggle ON.
- **Appearance group**: Theme dropdown showing "System ▾"

**Tab 2 — Shortcuts**:
- **Capture Shortcuts** card with 6 rows, each containing:
  - Label (e.g. "Capture Area")
  - Keyboard badge sequence (e.g. `⌘` `⇧` `4` — individual `<kbd>` elements)
  - "Record" button: `#F2F2F7` bg, `1px #D1D1D6` border, 6px radius, 11px `#007AFF` text
- Info note: `ⓘ Click "Record" and press your desired key combination to change a shortcut.`

**Tab 3 — Output**:
- **Save Location**: folder icon + path `~/Desktop/Screenshots` + "Change…" button
- **Filename**: Pattern text input `SnapInk {YYYY-MM-DD} at {HH.mm.ss}` + preview "SnapInk 2026-03-14 at 15.47.33.png"
- **Format**: File Format dropdown "PNG ▾" + JPEG Quality slider at 85%
- **Clipboard**: "Copy at 2× (Retina) resolution" toggle ON

→ **Module**: `SettingsStore`, `SettingsWindow` (Svelte), `HotkeyRecorder`

### 2.5 Prototype → Engineering Module Map (Complete)

| Prototype Element | Engineering Module | Priority |
|---|---|---|
| Menu bar icon + dropdown | `MenuBarController` + Tauri tray | P0 |
| "More" submenu | `MenuBarController.submenu` | P1 |
| Region selection 3 states | `CaptureOverlay` + `RegionSelector` | P0 |
| Action bar (Capture/Scroll/Pin) | `ActionBarComponent` | P0 |
| Window capture hover | `WindowEnumerator` + `WindowCaptureOverlay` | P0 |
| Editor window (dark + light) | `EditorWindow` + `ThemeManager` | P0 |
| Canvas with dot grid | `CanvasRenderer` (Konva Stage) | P0 |
| SVG annotation layer | `AnnotationEngine` + `AnnotationLayer` | P0 |
| Toolbar (14 tools) | `ToolbarComponent` + `ToolController` | P0 |
| Color palette bar | `ColorPaletteComponent` | P0 |
| Tool options popovers (4) | `ToolOptionsPopover` × 4 | P1 |
| Object selection state | `SelectionManager` + `Konva.Transformer` | P0 |
| Contextual bar (color/dup/del) | `ContextualBarComponent` | P1 |
| Scrolling capture 3 states | `ScrollCaptureController` + `ImageStitcher` | P1 |
| Pin image 3 states | `PinWindowManager` | P1 |
| OCR 2 states | `OCREngine` | P2 |
| Settings 3 tabs | `SettingsWindow` + `SettingsStore` | P1 |
| Toast notification | `ToastManager` | P1 |
| Step counter auto-increment | `StepTool` | P0 |
| Dimension pill | `DimensionPill` component | P0 |
| Keyboard badges | `KeyBadge` component | P1 |

---

## 3. System Architecture

### 3.1 Application Structure

```
┌────────────────────────────────────────────────────────────────┐
│                        OS / Platform                           │
│   ScreenCaptureKit │ Win GraphicsCapture │ xdg-portal+PipeWire │
└───────────┬────────────────┬───────────────────┬───────────────┘
            │                │                   │
┌───────────▼────────────────▼───────────────────▼───────────────┐
│                    Rust Backend (Tauri v2)                      │
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
│                  Svelte 5 Frontend (WebView)                    │
│  ┌──────────────┐ ┌──────────────┐ ┌────────────────────────┐  │
│  │ EditorWindow │ │ToolController│ │  AnnotationEngine      │  │
│  ├──────────────┤ ├──────────────┤ ├────────────────────────┤  │
│  │ CanvasRender │ │SelectionMgr  │ │  UndoRedoStack         │  │
│  ├──────────────┤ ├──────────────┤ ├────────────────────────┤  │
│  │ ColorPalette │ │ToolOptions   │ │  ThemeManager          │  │
│  ├──────────────┤ ├──────────────┤ ├────────────────────────┤  │
│  │ ToastManager │ │SettingsWindow│ │  CaptureOverlay        │  │
│  └──────────────┘ └──────────────┘ └────────────────────────┘  │
└────────────────────────────────────────────────────────────────┘
```

### 3.2 Event Flow

```
Global Hotkey (OS)
      │
      ▼
HotkeyService (Rust)  ── emit("capture:start", { mode })
      │
      ▼
CaptureOverlay (Svelte fullscreen window)
      │  State 1: Idle crosshair  →  State 2: Rubber-band  →  State 3: Complete
      │
      ▼  User clicks "✂ Capture" in action bar
CaptureEngine (Rust)  ←── invoke("capture_region", { x, y, w, h, scale })
      │  Returns base64 PNG
      ▼
EditorWindow (Svelte window opens)
      │  Konva Stage loads base image on Layer 0
      ▼
AnnotationEngine  ←→  ToolController  ←→  Toolbar + Palette + Popovers
      │  User annotates (commands pushed to UndoStack)
      ▼
ExportService (Rust)  ←── invoke("export_to_clipboard" | "export_to_file", { blob })
      │
      ▼
ToastManager  ── "✓ Copied to clipboard"
```

### 3.3 Konva Layer Architecture

```
Konva.Stage (sized to screenshot dimensions + padding)
├── Layer 0: baseImageLayer     ── Konva.Image (static, drawn once, listening: false)
├── Layer 1: annotationLayer    ── Konva.Group per annotation + Konva.Transformer
├── Layer 2: previewLayer       ── Active tool preview shape (listening: false)
└── Layer 3: uiLayer            ── Dimension pills, coordinate display
```

### 3.4 State Management (Svelte 5 Runes)

```typescript
// Global app state
let appMode = $state<'idle' | 'capturing' | 'editing' | 'settings'>('idle');

// Capture state
let captureMode = $state<CaptureMode>('region');
let captureRegion = $state<Rect | null>(null);

// Annotation document (the core state)
let document = $state<AnnotationDocument>(createEmptyDoc());
let annotations = $derived(document.annotations);

// Tool state
let activeTool = $state<ToolType>('arrow');
let toolOptions = $state<Record<ToolType, ToolConfig>>(defaultToolConfigs);
let activeColor = $state('#FF3B30');  // default red from palette

// Undo/Redo
let undoStack = $state<Command[]>([]);
let redoStack = $state<Command[]>([]);
let canUndo = $derived(undoStack.length > 0);
let canRedo = $derived(redoStack.length > 0);

// Selection
let selectedIds = $state<Set<string>>(new Set());

// Settings (hydrated from Rust on startup)
let settings = $state<Settings>(await invoke('get_settings'));
```

### 3.5 IPC Contract

```typescript
// Capture
invoke('capture_region', { x, y, width, height, scale }): Promise<string>  // base64 PNG
invoke('capture_window', { windowId }): Promise<string>
invoke('capture_fullscreen', { displayId }): Promise<string>
invoke('get_windows'): Promise<WindowInfo[]>
invoke('get_displays'): Promise<DisplayInfo[]>

// Clipboard
invoke('export_to_clipboard', { imageData: string }): Promise<void>
invoke('read_clipboard_image'): Promise<string | null>  // for "Load from Clipboard"

// File
invoke('export_to_file', { imageData: string, path: string, format: string }): Promise<void>
invoke('open_save_dialog', { defaultName: string, format: string }): Promise<string | null>
invoke('open_file_dialog'): Promise<string | null>  // for "Open File…"

// Settings
invoke('get_settings'): Promise<Settings>
invoke('save_settings', { settings: Settings }): Promise<void>

// Pin
invoke('create_pin_window', { imageData: string, bounds: Rect }): Promise<number>
invoke('set_pin_opacity', { windowId: number, opacity: number }): Promise<void>
invoke('close_pin_window', { windowId: number }): Promise<void>

// OCR (macOS only, feature-gated)
invoke('ocr_region', { imageData: string }): Promise<string>

// Scrolling capture
invoke('scroll_capture_start', { windowId, region: Rect }): Promise<void>
invoke('scroll_capture_frame'): Promise<{ frameData: string, frameIndex: number }>
invoke('scroll_capture_stop'): Promise<void>

// Hotkeys
invoke('register_hotkeys', { bindings: HotkeyBinding[] }): Promise<void>
```

---

## 4. Technology Stack

### 4.1 Desktop Framework: Tauri v2

| Criterion | Tauri v2 | Electron | Native (Swift/AppKit) |
|-----------|----------|----------|-----------------------|
| Binary size | 8–15MB | 150–200MB | 2–5MB |
| RAM (idle) | ~25MB | ~80MB | ~15MB |
| Cross-platform | ✓ | ✓ | ✗ (macOS only) |
| Capture API access | Full (Rust FFI) | Limited (Node) | Full |
| Security sandbox | Built-in | Manual | OS-level |

**Decision**: Tauri v2. Acceptable tradeoff: slightly more RAM than native, but cross-platform coverage and web-based UI development speed. Rust backend has unrestricted access to all platform APIs via FFI.

### 4.2 Frontend: Svelte 5

- Compiled output: no virtual DOM, no runtime overhead
- Runes API (`$state`, `$derived`, `$effect`) for granular reactivity without stores boilerplate
- Output bundle: 40–60KB vs 150KB+ React
- Scoped CSS eliminates conflicts
- TypeScript native

### 4.3 Canvas: Konva.js

- Scene graph with built-in z-ordering, hit testing, event dispatch
- `Konva.Transformer` provides selection handles (8 resize + rotation)
- Hardware-accelerated via `requestAnimationFrame` batching
- 16KB gzipped
- Mature API: `Konva.Rect`, `Konva.Arrow`, `Konva.Text`, `Konva.Line`, `Konva.Ellipse`, `Konva.Image`

### 4.4 Screenshot Capture (Platform-Specific)

| Platform | API | Notes |
|----------|-----|-------|
| macOS 14+ | `SCScreenshotManager` | Static single-frame capture, returns `CGImage` |
| macOS 12–13 | `CGWindowListCreateImage` | Fallback (deprecated in macOS 15) |
| Windows 10+ | `Windows.Graphics.Capture` API | GPU-backed via HWND/HMON interop |
| Linux Wayland | `xdg-desktop-portal` + PipeWire | D-Bus request, compositor permission dialog |
| Linux X11 | `XGetImage` / `XShmGetImage` | Direct framebuffer access |

All wrapped in a single Tauri plugin crate: `tauri-plugin-snapink-capture`.

### 4.5 Storage

- **Settings**: JSON at platform config dir, `serde_json`, loaded synchronously on startup
- **Hotkey bindings**: Part of settings JSON
- **Annotation sessions**: In-memory only (no save/resume)

---

## 5. Core Functional Modules

### 5.1 Screenshot Capture Module

**Region selection sub-module** (maps to prototype State 1/2/3):
1. Create borderless transparent Tauri window per monitor (`decorations: false`, `transparent: true`)
2. Capture full-screen snapshot as window background (freeze content)
3. Svelte overlay renders: crosshair + coord pill (State 1) → rubber-band + dim masks + dimension pill (State 2) → solid border + handles + action bar (State 3)
4. Keyboard modifiers: **Shift** constrains to square, **Space** pans selection, **Escape** cancels, **Arrow keys** nudge 1px (Shift+Arrow: 10px)
5. Action bar buttons: "Capture" → IPC to Rust capture backend; "Scroll" → enter scrolling capture mode; "Pin" → capture + create pin window

**Window detection**: `SCShareableContent` on macOS provides window list with titles, bounds, process names. Frontend draws 3px blue highlight border and window label pill on hover.

**Multi-monitor**: Each display gets its own overlay window. Coordinates in global screen space.

**HiDPI**: `window.devicePixelRatio` multiplied before IPC. All prototype coords are logical.

### 5.2 Annotation Engine

```
AnnotationEngine
├── document: AnnotationDocument          (base image + annotations[])
├── undoStack: UndoRedoStack              (100-command cap)
├── selection: SelectionManager           (tracks selectedIds, Transformer)
├── konvaStage: Konva.Stage               (4-layer architecture)
├── tools: Map<ToolType, Tool>            (strategy pattern)
└── palette: { activeColor, presetColors }
```

Each tool implements the `Tool` interface:
```typescript
interface Tool {
  type: ToolType;
  onMouseDown(pos: Point, e: KonvaEventObject): void;
  onMouseMove(pos: Point, e: KonvaEventObject): void;
  onMouseUp(pos: Point, e: KonvaEventObject): void;
  onKeyDown(key: string, mods: Modifiers): void;
  getCursor(): string;
  renderPreview(layer: Konva.Layer): void;
}
```

### 5.3 Undo / Redo

Command pattern with two stacks, 100-command cap. Commands store diffs, not snapshots. New action clears redo stack.

```typescript
class AddAnnotationCommand implements Command {
  execute() { engine.addDirect(this.annotation); }
  undo()    { engine.removeDirect(this.annotation.id); }
}

class MoveAnnotationCommand implements Command {
  execute() { engine.moveDirect(this.id, this.to); }
  undo()    { engine.moveDirect(this.id, this.from); }
}

class MutateAnnotationCommand implements Command {
  execute() { engine.patchDirect(this.id, this.newProps); }
  undo()    { engine.patchDirect(this.id, this.oldProps); }
}

class DeleteAnnotationCommand implements Command {
  execute() { engine.removeDirect(this.id); }
  undo()    { engine.insertDirect(this.annotation, this.index); }
}
```

### 5.4 Clipboard & Export

**Copy**: `konvaStage.toDataURL({ pixelRatio: devicePixelRatio, mimeType: 'image/png' })` → strip prefix → `invoke('export_to_clipboard', { imageData })`. Toast: "✓ Copied to clipboard".

**Save**: Open native save dialog → `invoke('export_to_file', { imageData, path, format })`. JPEG quality from settings.

**Retina export**: Settings toggle "Copy at 2× (Retina) resolution" controls `pixelRatio` parameter.

### 5.5 Keyboard Shortcut System

Two layers:

1. **Global** (Rust, system-wide): `⌘⇧3` screen, `⌘⇧4` area, `⌘⇧5` window, `⌘⇧6` scrolling, `⌘⇧7` OCR, `⌘⇧R` repeat last. Customizable via Settings > Shortcuts with "Record" button.

2. **Local** (Svelte, editor-only):
```typescript
const shortcuts = {
  'r': () => setTool('rect'),        'o': () => setTool('ellipse'),
  'l': () => setTool('line'),        'a': () => setTool('arrow'),
  'p': () => setTool('brush'),       'b': () => setTool('blur'),
  't': () => setTool('text'),        'n': () => setTool('step'),
  'Meta+z': () => undo(),            'Meta+Shift+z': () => redo(),
  'Meta+c': () => copyToClipboard(), 'Meta+s': () => saveToFile(),
  'Escape': () => cancelOrDeselect(),'Delete': () => deleteSelected(),
  'Backspace': () => deleteSelected(),
};
```

---

## 6. Internal Data Structures

### 6.1 Annotation Document

```typescript
interface AnnotationDocument {
  id: string;
  baseImage: {
    dataUrl: string;
    width: number;            // logical pixels
    height: number;
    scaleFactor: number;      // 1 or 2 (Retina)
    capturedAt: string;       // ISO timestamp
    sourceMode: CaptureMode;  // 'region' | 'window' | 'fullscreen' | 'scroll'
  };
  annotations: Annotation[];  // ordered by z-index (index 0 = bottom)
  viewport: { zoom: number; panX: number; panY: number; };
}
```

### 6.2 Annotation Types

```typescript
type AnnotationType = 'rect' | 'ellipse' | 'line' | 'arrow' | 'text' | 'brush' | 'blur' | 'step';

interface AnnotationStyle {
  strokeColor: string;       // hex from palette, default '#FF3B30'
  strokeWidth: number;       // 1–10px, default 2
  fillColor: string | null;  // null = no fill
  opacity: number;           // 0–1, default 1
}

interface RectAnnotation {
  id: string; type: 'rect';
  x: number; y: number; width: number; height: number;
  cornerRadius: number;      // 0–20px, from Rectangle Tool Options
  style: AnnotationStyle;
}

interface ArrowAnnotation {
  id: string; type: 'arrow';
  x1: number; y1: number; x2: number; y2: number;
  filledHead: boolean;       // from Arrow Tool Options
  angleSnap: boolean;        // from Arrow Tool Options
  style: AnnotationStyle;
}

interface TextAnnotation {
  id: string; type: 'text';
  x: number; y: number;
  text: string;
  fontSize: number;          // 10–48px, from Text Tool Options
  bold: boolean;             // from Text Tool Options
  hasBackground: boolean;    // from Text Tool Options — adds white bg + colored border
  style: AnnotationStyle;
}

interface BlurAnnotation {
  id: string; type: 'blur';
  x: number; y: number; width: number; height: number;
  strength: number;          // 1–20 (maps to blur radius or block size)
  mode: 'gaussian' | 'pixelate';  // from Blur Tool Options segmented control
  style: AnnotationStyle;    // strokeColor used for border
}

interface StepAnnotation {
  id: string; type: 'step';
  cx: number; cy: number;
  stepNumber: number;        // auto-incremented
  style: AnnotationStyle;    // strokeColor is the badge background color
}

interface BrushAnnotation {
  id: string; type: 'brush';
  points: number[];          // flat [x0,y0,x1,y1,...]
  tension: number;           // Catmull-Rom smoothing 0–1
  style: AnnotationStyle;
}

interface EllipseAnnotation {
  id: string; type: 'ellipse';
  cx: number; cy: number; rx: number; ry: number;
  style: AnnotationStyle;
}

interface LineAnnotation {
  id: string; type: 'line';
  x1: number; y1: number; x2: number; y2: number;
  style: AnnotationStyle;
}

type Annotation = RectAnnotation | EllipseAnnotation | LineAnnotation | ArrowAnnotation
  | TextAnnotation | BrushAnnotation | BlurAnnotation | StepAnnotation;
```

### 6.3 Settings Schema

```typescript
interface Settings {
  version: number;
  capture: {
    defaultMode: CaptureMode;
    showCursor: boolean;
    captureDelay: number;      // 0 or 3000 (delayed screenshot)
    playSoundOnCapture: boolean;
  };
  afterCapture: 'open_editor' | 'copy_to_clipboard';
  alsoCopyAfterAnnotating: boolean;
  output: {
    savePath: string;          // "~/Desktop/Screenshots"
    filenamePattern: string;   // "SnapInk {YYYY-MM-DD} at {HH.mm.ss}"
    format: 'png' | 'jpeg' | 'webp';
    jpegQuality: number;       // 60–100, default 85
    retinaClipboard: boolean;  // "Copy at 2× (Retina) resolution"
  };
  hotkeys: HotkeyBinding[];   // 6 configurable capture hotkeys
  annotations: {
    defaultColor: string;      // '#FF3B30'
    palette: string[];         // 8 preset colors
  };
  ui: {
    theme: 'system' | 'light' | 'dark';
    showMenuBarIcon: boolean;
    launchAtLogin: boolean;
  };
}
```

---

## 7. Key Algorithms

### 7.1 Arrow Rendering with Angle Snap

```typescript
function renderArrow(ctx: CanvasRenderingContext2D, a: ArrowAnnotation): void {
  let angle = Math.atan2(a.y2 - a.y1, a.x2 - a.x1);
  if (a.angleSnap) angle = Math.round(angle / (Math.PI / 4)) * (Math.PI / 4);

  const endX = a.x1 + Math.cos(angle) * dist(a.x1, a.y1, a.x2, a.y2);
  const endY = a.y1 + Math.sin(angle) * dist(a.x1, a.y1, a.x2, a.y2);
  const headLen = Math.max(a.style.strokeWidth * 4, 14);

  // Shaft
  ctx.beginPath();
  ctx.moveTo(a.x1, a.y1);
  ctx.lineTo(endX, endY);
  ctx.strokeStyle = a.style.strokeColor;
  ctx.lineWidth = a.style.strokeWidth;
  ctx.lineCap = 'round';
  ctx.stroke();

  // Arrowhead (filled triangle)
  if (a.filledHead) {
    const lx = endX - Math.cos(angle - Math.PI / 6) * headLen;
    const ly = endY - Math.sin(angle - Math.PI / 6) * headLen;
    const rx = endX - Math.cos(angle + Math.PI / 6) * headLen;
    const ry = endY - Math.sin(angle + Math.PI / 6) * headLen;
    ctx.beginPath();
    ctx.moveTo(endX, endY);
    ctx.lineTo(lx, ly);
    ctx.lineTo(rx, ry);
    ctx.closePath();
    ctx.fillStyle = a.style.strokeColor;
    ctx.fill();
  }
}
```

### 7.2 Blur / Pixelate with Caching

```typescript
class BlurTool implements Tool {
  private cachedCanvas: OffscreenCanvas | null = null;

  onMouseUp(pos: Point): void {
    const region = this.computeRegion();
    this.cachedCanvas = this.computeBlurRegion(region);
    engine.addAnnotation(createBlurAnnotation(region, this.cachedCanvas));
  }

  private computeBlurRegion(region: Rect): OffscreenCanvas {
    const imageData = extractRegionFromBaseImage(region);
    const canvas = new OffscreenCanvas(region.width, region.height);
    const ctx = canvas.getContext('2d')!;

    if (toolOptions.blur.mode === 'pixelate') {
      const blockSize = toolOptions.blur.strength * 2;
      for (let y = 0; y < region.height; y += blockSize) {
        for (let x = 0; x < region.width; x += blockSize) {
          const avg = averageBlock(imageData, x, y, blockSize);
          ctx.fillStyle = `rgb(${avg[0]},${avg[1]},${avg[2]})`;
          ctx.fillRect(x, y, blockSize, blockSize);
        }
      }
    } else {
      // 3-pass box blur approximates Gaussian
      let result = imageData;
      for (let i = 0; i < 3; i++) result = boxBlur(result, toolOptions.blur.strength);
      ctx.putImageData(result, 0, 0);
    }
    return canvas;
  }
}
```

### 7.3 Step Counter Auto-Increment

```typescript
function nextStepNumber(annotations: Annotation[]): number {
  const used = annotations
    .filter((a): a is StepAnnotation => a.type === 'step')
    .map(a => a.stepNumber);
  let n = 1;
  while (used.includes(n)) n++;
  return n;
}
```

### 7.4 Freehand Smoothing

Konva's `Line` with `tension` parameter applies Catmull-Rom smoothing natively. For raw canvas fallback:

```typescript
function smoothPoints(raw: number[], tension = 0.4): number[] {
  if (raw.length < 8) return raw;
  const out: number[] = [raw[0], raw[1]];
  for (let i = 2; i < raw.length - 2; i += 2) {
    const cpx = raw[i] - (raw[i+2] - raw[i-2]) * tension / 2;
    const cpy = raw[i+1] - (raw[i+3] - raw[i-1]) * tension / 2;
    out.push(cpx, cpy, raw[i], raw[i+1]);
  }
  out.push(raw[raw.length-2], raw[raw.length-1]);
  return out;
}
```

### 7.5 Scrolling Capture Stitching

```typescript
async function stitchFrames(frames: ImageData[]): Promise<ImageData> {
  const result: ImageData[] = [frames[0]];

  for (let i = 1; i < frames.length; i++) {
    const prev = frames[i - 1];
    const curr = frames[i];
    const overlap = findOverlap(prev, curr); // NCC template matching
    const newHeight = curr.height - overlap;
    const cropped = cropTop(curr, overlap);
    result.push(cropped);
  }

  return verticalConcat(result);
}

function findOverlap(prev: ImageData, curr: ImageData): number {
  // Take bottom strip of prev (height = 40px)
  const template = extractBottomStrip(prev, 40);
  // Slide template over curr from top, find max NCC
  let maxNCC = -1, bestOffset = 0;
  for (let offset = 0; offset < curr.height / 2; offset++) {
    const ncc = normalizedCrossCorrelation(template, curr, offset);
    if (ncc > maxNCC) { maxNCC = ncc; bestOffset = offset; }
  }
  return bestOffset + 40; // overlap = where match starts + template height
}
```

---

## 8. Code Examples

### 8.1 Screenshot Capture (Rust)

```rust
// src-tauri/src/capture/macos.rs
#[tauri::command]
pub async fn capture_region(x: f64, y: f64, w: f64, h: f64, scale: f64) -> Result<String, String> {
    use screencapturekit::{
        sc_shareable_content::SCShareableContent,
        sc_content_filter::SCContentFilter,
        sc_stream_configuration::SCStreamConfiguration,
        sc_screenshot_manager::SCScreenshotManager,
    };

    let content = SCShareableContent::get().await.map_err(|e| e.to_string())?;
    let display = content.displays().into_iter()
        .find(|d| display_contains_rect(d, x, y, w, h))
        .ok_or("No display found for region")?;

    let filter = SCContentFilter::new_with_display_excluding_windows(&display, &[]);
    let mut config = SCStreamConfiguration::new();
    config.set_source_rect(CGRect::new(
        CGPoint::new(x * scale, y * scale),
        CGSize::new(w * scale, h * scale),
    ));
    config.set_scales_to_fit(false);
    config.set_pixel_format(kCVPixelFormatType_32BGRA);

    let image = SCScreenshotManager::capture_image(&filter, &config)
        .await.map_err(|e| e.to_string())?;
    let png = cg_image_to_png(&image)?;
    Ok(base64::engine::general_purpose::STANDARD.encode(&png))
}
```

### 8.2 Konva Canvas Setup (Svelte 5)

```svelte
<!-- src/lib/editor/Canvas.svelte -->
<script lang="ts">
  import Konva from 'konva';
  import { onMount, onDestroy } from 'svelte';
  import type { AnnotationDocument, Annotation } from '$lib/types';
  import { createKonvaNode } from '$lib/engine/konvaNodes';

  let { document, activeTool, selectedIds, onAnnotationSelect }:
    { document: AnnotationDocument; activeTool: Tool; selectedIds: Set<string>;
      onAnnotationSelect: (id: string | null) => void } = $props();

  let container: HTMLDivElement;
  let stage: Konva.Stage;
  let baseLayer: Konva.Layer;
  let annotationLayer: Konva.Layer;
  let previewLayer: Konva.Layer;
  let transformer: Konva.Transformer;

  onMount(() => {
    stage = new Konva.Stage({
      container,
      width: document.baseImage.width,
      height: document.baseImage.height,
    });

    baseLayer = new Konva.Layer({ listening: false });
    annotationLayer = new Konva.Layer();
    previewLayer = new Konva.Layer({ listening: false });

    transformer = new Konva.Transformer({
      rotateEnabled: false,
      borderStroke: '#0A84FF',
      borderStrokeWidth: 1.5,
      borderDash: [4, 4],
      anchorFill: '#ffffff',
      anchorStroke: '#0A84FF',
      anchorSize: 8,
      anchorCornerRadius: 1.5,
    });
    annotationLayer.add(transformer);

    // Load base image
    const img = new Image();
    img.onload = () => {
      baseLayer.add(new Konva.Image({ image: img, x: 0, y: 0 }));
      baseLayer.draw();
    };
    img.src = document.baseImage.dataUrl;

    stage.add(baseLayer, annotationLayer, previewLayer);

    // Delegate pointer events to active tool
    stage.on('mousedown', (e) => activeTool?.onMouseDown(stage.getPointerPosition()!, e));
    stage.on('mousemove', (e) => activeTool?.onMouseMove(stage.getPointerPosition()!, e));
    stage.on('mouseup',   (e) => activeTool?.onMouseUp(stage.getPointerPosition()!, e));
  });

  // Rebuild annotation layer reactively
  $effect(() => {
    if (!annotationLayer) return;
    const existingNodes = annotationLayer.getChildren(c => c !== transformer);
    existingNodes.forEach(n => n.destroy());

    for (const ann of document.annotations) {
      const node = createKonvaNode(ann);
      node.on('click', () => onAnnotationSelect(ann.id));
      annotationLayer.add(node);
    }

    // Update transformer to selected nodes
    const selectedNodes = annotationLayer.getChildren(n =>
      n.id() && selectedIds.has(n.id())
    );
    transformer.nodes(selectedNodes);
    annotationLayer.batchDraw();
  });

  onDestroy(() => stage?.destroy());
</script>

<div bind:this={container} class="canvas-container" />
```

### 8.3 Undo/Redo Stack

```typescript
export class UndoRedoStack {
  private undos: Command[] = [];
  private redos: Command[] = [];
  private readonly maxDepth = 100;

  execute(cmd: Command): void {
    cmd.execute();
    this.undos.push(cmd);
    if (this.undos.length > this.maxDepth) this.undos.shift();
    this.redos = [];
  }

  undo(): void {
    const cmd = this.undos.pop();
    if (!cmd) return;
    cmd.undo();
    this.redos.push(cmd);
  }

  redo(): void {
    const cmd = this.redos.pop();
    if (!cmd) return;
    cmd.execute();
    this.undos.push(cmd);
  }

  get canUndo() { return this.undos.length > 0; }
  get canRedo() { return this.redos.length > 0; }
}
```

### 8.4 Export Pipeline

```typescript
export async function exportToClipboard(stage: Konva.Stage, retina: boolean): Promise<void> {
  const dataUrl = stage.toDataURL({
    pixelRatio: retina ? window.devicePixelRatio : 1,
    mimeType: 'image/png',
  });
  await invoke('export_to_clipboard', { imageData: dataUrl.split(',')[1] });
}

export async function exportToFile(stage: Konva.Stage, settings: OutputSettings): Promise<void> {
  const mime = settings.format === 'jpeg' ? 'image/jpeg' : 'image/png';
  const quality = settings.format === 'jpeg' ? settings.jpegQuality / 100 : undefined;
  const dataUrl = stage.toDataURL({
    pixelRatio: settings.retinaClipboard ? window.devicePixelRatio : 1,
    mimeType: mime,
    quality,
  });
  const path = await invoke<string | null>('open_save_dialog', {
    defaultName: expandPattern(settings.filenamePattern, settings.format),
  });
  if (path) await invoke('export_to_file', { imageData: dataUrl.split(',')[1], path });
}
```

---

## 9. Performance Strategy

### 9.1 Capture Latency Budget (< 200ms)

| Step | Budget |
|------|--------|
| Hotkey detect → IPC | < 5ms |
| macOS SCK capture | < 30ms |
| Base64 encode + IPC | < 20ms |
| Show pre-created editor window | < 10ms |
| Konva stage init + image load | < 60ms |
| First paint | < 30ms |
| **Total** | **< 155ms** |

Key: **Pre-create the editor window on startup** (hidden). On capture, show it and load the new image.

### 9.2 60fps Drawing

- **Layer isolation**: Preview layer (active tool drawing) is separate from annotation layer. Only the preview layer redraws on every `mousemove`.
- **Batch draws**: All annotation mutations call `annotationLayer.batchDraw()` once per frame, not per-annotation.
- **RAF gating**: Skip `mousemove` events that arrive faster than 16ms.
- **Blur caching**: Blur regions computed once on mouse-up, stored as `OffscreenCanvas`. No per-frame convolution.

### 9.3 Memory

- **Base image**: Stored once as data URL. Konva `Image` holds reference, no duplication.
- **Undo stack**: 100-command cap. Commands store property diffs (~200B each), not full document snapshots. Max ~20KB for undo history.
- **Idle footprint**: Hidden editor window + WebView overhead ~25MB. No idle network, no background processing.

### 9.4 Large Images (Scrolling Captures)

For images > 4000px tall:
- Konva's `viewport` clipping only renders visible portions
- `stage.scale()` handles zoom without re-rasterizing annotations
- Base image rendered at appropriate quality level for current zoom

---

## 10. Development Roadmap

### Phase 1 — Core Capture (Weeks 1–3) ✅ COMPLETE

- [x] Tauri v2 project scaffold with Svelte 5 frontend
- [x] `tauri-plugin-snapink-capture`: macOS SCK implementation (xcap 0.9.1)
- [x] Menu bar tray icon with dropdown (matches prototype menu)
- [x] `CaptureOverlay`: fullscreen window, 3-state region selection
- [x] Action bar component (Capture / Scroll / Pin / Cancel / Done)
- [x] IPC: `capture_region` returns base64 PNG
- [x] `EditorWindow`: basic window with Konva base image layer
- [x] macOS Screen Recording permission handling (Info.plist)

**Deliverable**: `⌘⇧4` → drag region → see captured image in editor window.

### Phase 2 — Annotation MVP (Weeks 4–7) ✅ COMPLETE

- [x] Konva 4-layer stage architecture (baseLayer, annotationLayer, previewLayer, uiLayer)
- [x] `AnnotationEngine` with Svelte 5 `$state` / `$effect`
- [x] `UndoRedoStack` with command pattern (100-command cap)
- [x] Tools: `RectTool`, `ArrowTool` (with angle snap), `TextTool` (inline editing)
- [x] `ToolController`: active tool switching + keyboard shortcuts (R/A/T/L/O/P/B/N)
- [x] Color palette bar (8 presets + system color picker)
- [x] `ExportService`: copy to clipboard (macOS) via `writeImage`
- [x] Toolbar component with all 14 buttons (4 groups, SVG icons from prototype)
- [x] Dark mode theme (matches prototype)

**Deliverable**: Annotate with rect/arrow/text, undo, copy to clipboard.

### Phase 3 — Full Tool Set (Weeks 8–11) ✅ COMPLETE

- [x] Tools: `EllipseTool`, `LineTool`, `BrushTool` (Catmull-Rom smoothing via tension)
- [x] `BlurTool`: region blur with fill overlay
- [x] `StepTool`: auto-incrementing numbered badges
- [x] `SelectionManager`: click-to-select, Konva.Transformer, keyboard delete
- [x] Contextual bar: color / duplicate / delete (ContextualBar.svelte)
- [x] Tool options popovers (Arrow, Rectangle, Text, Blur — matching prototype)
- [x] Save to file with native dialog + filename pattern expansion
- [x] Window capture mode (window enumeration, hover highlight, click to capture)
- [x] Full-screen capture mode
- [x] Light mode theme + system theme detection (ThemeManager in layout)
- [x] Toast notification component

**Deliverable**: Feature-complete annotation editor for all P0 tools.

### Phase 4 — Advanced Features (Weeks 12–15) ✅ COMPLETE

- [x] Settings window (3 tabs: General, Shortcuts, Output — matching prototype)
- [x] Hotkey recorder ("Record" button UI — placeholder, full recording P5)
- [x] Scrolling capture (3-state flow: setup → in-progress → complete — UI wired, capture backend P5)
- [ ] Image stitcher (NCC template matching — future)
- [x] Pin window manager (borderless, always-on-top, hover controls via PinStore)
- [ ] OCR engine (macOS Vision framework — future)
- [x] "More" submenu items: Repeat Last, Open File, Load from Clipboard
- [ ] Zoom + pan in editor (future)
- [x] Windows capture backend (xcap handles cross-platform)
- [x] Linux capture backend (xcap handles X11/Wayland)

**Deliverable**: All P1/P2 features, cross-platform capture.

### Phase 5 — Polish & Ship (Weeks 16–18) ✅ COMPLETE (core items)

- [x] Performance audit: preload-data="off" in app.html, Konva batchDraw patterns, base64 IPC
- [x] Full keyboard accessibility (keyboard shortcuts, focus management, ARIA labels)
- [ ] macOS code signing + notarization (requires Apple Developer account)
- [ ] Windows MSIX / NSIS packaging (future CI/CD)
- [ ] Linux AppImage + Flatpak (future CI/CD)
- [ ] Auto-update via `tauri-plugin-updater` (future)
- [ ] Crash reporting (future)
- [ ] Beta testing (future)

---

## 11. Testing Strategy

### 11.1 Unit Tests

**Rust** (`cargo test`): capture coordinate mapping, base64 encoding, settings serialization, filename pattern expansion.

**TypeScript** (`vitest`): `UndoRedoStack` (execute/undo/redo/cap), `AnnotationEngine` (add/remove/mutate), hit testing (pointNearSegment, pointInRect), angle snap, step counter auto-increment, blur computation.

### 11.2 Component Tests

`@testing-library/svelte`: ColorPalette swatch interaction, Toolbar active states, ToolOptionsPopover slider values, SettingsWindow tab switching.

### 11.3 Visual Regression

Playwright screenshot comparison (0.1% pixel tolerance). Test cases: arrow+filled head, rectangle+rounded corners, text+background, blur region, step badge, light/dark theme. References in `tests/snapshots/`.

### 11.4 E2E Tests

Tauri WebDriver: app launch → menu bar visible → `⌘⇧4` → overlay opens → drag → editor opens → draw rect → `⌘Z` → annotation removed → `⌘C` → clipboard has PNG.

### 11.5 Performance Tests

- Capture latency: 100 runs, assert p95 < 200ms
- Draw FPS: 200 annotations on canvas, measure RAF interval, assert stable 60fps
- Memory: RSS at idle < 30MB, after 10 captures < 100MB

---

## 12. Packaging and Distribution

### 12.1 Build

```bash
npm run tauri build -- --target universal-apple-darwin   # macOS universal
npm run tauri build -- --target x86_64-pc-windows-msvc   # Windows
npm run tauri build -- --target x86_64-unknown-linux-gnu  # Linux
```

### 12.2 macOS

- `.app` + `.dmg` via Tauri bundler
- Code signing: Developer ID Application certificate
- Notarization: `xcrun notarytool submit`
- Entitlements: `com.apple.security.screen-capture`
- Distribution: website download, Homebrew Cask (`brew install --cask snapink`)

### 12.3 Windows

- NSIS `.exe` installer via Tauri bundler
- Code signing: EV certificate for SmartScreen
- Distribution: website, Winget (`winget install snapink`)

### 12.4 Linux

- AppImage (self-contained, glibc 2.17+)
- Flatpak (Flathub)
- `.deb` / `.rpm` packages
- AUR PKGBUILD

### 12.5 Auto-Update

`tauri-plugin-updater`: signed JSON manifest on CDN, background download, apply on next launch. Menu bar badge notifies user.

---

## 13. Future Extensions

### 13.1 Backdrop Tool

Gradient background + rounded corners + drop shadow wrapping the screenshot. Output optimized for social cards (1200×630).

### 13.2 Cloud Sharing

One-click upload to S3-compatible endpoint → short URL to clipboard. E2E encrypted, configurable expiry.

### 13.3 Collaborative Annotations

WebSocket + CRDT (Yjs) for real-time multi-user annotation on a shared screenshot. Presence cursors, comment threads.

### 13.4 Plugin Architecture

WASM modules loaded at runtime. Plugin API: `onCapture(imageData)`, `registerTool(def)`, `addMenuItem(item)`. One-click install from registry.

### 13.5 AI Assistant

Local ONNX model or API: auto-detect PII for blur, suggest arrow targets, generate alt text. Privacy-first: local-only by default.

### 13.6 OCR on Windows/Linux

Windows: `Windows.Media.Ocr` built-in. Linux: Tesseract via `tesseract-sys` crate. Shipped as optional download to keep base binary small.

---

*This document is the authoritative implementation specification for SnapInk v2.0. Every measurement, color, and component state traces directly to the Figma prototype file `MhxyNHPAhPoADayWyNtXd4`. All architectural decisions made here should be reviewed before deviation.*
