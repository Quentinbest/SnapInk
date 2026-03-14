# SnapInk — Product Prototype Design Implementation Plan

> Figma prototype design plan for SnapInk, a macOS-native screenshot annotation tool.

---

## Table of Contents

1. [Product Vision & Scope](#1-product-vision--scope)
2. [Figma Project Structure](#2-figma-project-structure)
3. [Design System Foundation](#3-design-system-foundation)
4. [Screen Inventory & User Flows](#4-screen-inventory--user-flows)
5. [Page 1: Menu Bar & Trigger](#5-page-1-menu-bar--trigger)
6. [Page 2: Region Selection Overlay](#6-page-2-region-selection-overlay)
7. [Page 3: Window Capture](#7-page-3-window-capture)
8. [Page 4: Annotation Editor](#8-page-4-annotation-editor)
9. [Page 5: Annotation Tools — Detail Specs](#9-page-5-annotation-tools--detail-specs)
10. [Page 6: Scrolling Capture](#10-page-6-scrolling-capture)
11. [Page 7: Pin Image (Floating Window)](#11-page-7-pin-image-floating-window)
12. [Page 8: Settings / Preferences](#12-page-8-settings--preferences)
13. [Interactive Prototype Wiring](#13-interactive-prototype-wiring)
14. [Component Library Checklist](#14-component-library-checklist)
15. [Design Deliverables & Handoff](#15-design-deliverables--handoff)
16. [Implementation Phases](#16-implementation-phases)

---

## 1. Product Vision & Scope

### 1.1 What is SnapInk

SnapInk is a lightweight, fast, macOS-native screenshot annotation tool. It lives in the menu bar and enables users to capture, annotate, and share screenshots with minimal friction.

### 1.2 Target Users

- Developers documenting bugs, PRs, and code reviews
- Designers communicating UI feedback
- Product managers creating specs and walkthroughs
- Technical writers producing step-by-step tutorials

### 1.3 Prototype Scope (MVP)

The Figma prototype covers the **complete end-to-end user journey** for the following feature set:

| Feature | Priority | Notes |
|---------|----------|-------|
| Menu bar app trigger | P0 | Entry point for all actions |
| Area capture (region selection) | P0 | Crosshair + rubber-band + dimension display |
| Window capture | P0 | Hover-highlight + click to capture |
| Full-screen capture | P0 | Single shortcut, no selection UI |
| Annotation editor | P0 | Core editing canvas with toolbar |
| Annotation tools (shapes, arrow, text, blur, step counter) | P0 | Full toolbar |
| Copy to clipboard / Save | P0 | Export actions |
| Scrolling capture | P1 | Scroll-and-stitch flow |
| Pin image (floating window) | P1 | Always-on-top reference |
| OCR text recognition | P2 | Select region → extract text |
| Color picker (screen) | P2 | Eyedropper from live screen |
| Backdrop / beautification | P2 | Gradient bg, rounded corners, shadow |
| Settings / Preferences | P1 | Hotkeys, save path, behavior |

### 1.4 Design Principles

1. **Speed-first**: Capture → annotate → share in under 5 seconds for simple cases
2. **Invisible until needed**: Menu bar only, no dock icon, minimal chrome
3. **Inline editing**: Annotations happen directly on the capture, no separate app window
4. **macOS-native feel**: Follow Apple HIG, use system colors, blur materials, SF Symbols
5. **Non-destructive**: All annotations are objects that can be selected, moved, deleted

---

## 2. Figma Project Structure

### 2.1 File Organization

```
SnapInk (Figma File)
├── 📄 Cover Page              — Project title, version, status, links
├── 📄 Design System           — Colors, typography, icons, components
├── 📄 User Flows              — Flow diagrams connecting all screens
├── 📄 Menu Bar & Trigger      — Menu bar dropdown, hotkey hints
├── 📄 Region Selection        — Fullscreen overlay, crosshair, rubber-band
├── 📄 Window Capture          — Window detection, highlight, shadow options
├── 📄 Annotation Editor       — Main editor canvas with toolbar
├── 📄 Tool Detail Specs       — Each annotation tool's interaction states
├── 📄 Scrolling Capture       — Scroll flow, stitching preview
├── 📄 Pin Image               — Floating window variants
├── 📄 Settings                — Preferences window
└── 📄 Prototype Flows         — Wired interactive prototype frames
```

### 2.2 Naming Conventions

- **Frames**: `[Page]/[Section]/[State]` — e.g., `Editor/Toolbar/Arrow-Selected`
- **Components**: `[Category]/[Name]/[Variant]` — e.g., `Tool-Button/Arrow/Default`, `Tool-Button/Arrow/Active`
- **Variables**: Use Figma variables for colors, spacing, radii (enables theme switching)
- **Auto Layout**: Use on all components for responsive behavior

### 2.3 Recommended Figma Plugins

| Plugin | Purpose |
|--------|---------|
| **SF Symbols** | macOS-native icon set |
| **Phosphor Icons** | Fallback icon set for annotation tools |
| **Contrast** | Verify accessibility of color combinations |
| **Autoflow** | Visualize prototype connections between frames |

---

## 3. Design System Foundation

### 3.1 Color Palette

Define as **Figma Variables** with light/dark mode variants.

#### Core UI Colors (macOS-aligned)

| Variable Name | Light Mode | Dark Mode | Usage |
|---------------|------------|-----------|-------|
| `bg/primary` | `#FFFFFF` | `#1E1E1E` | App surfaces, editor bg |
| `bg/overlay` | `rgba(0,0,0,0.40)` | `rgba(0,0,0,0.55)` | Region selection dimming |
| `bg/toolbar` | `rgba(255,255,255,0.85)` | `rgba(40,40,40,0.85)` | Toolbar background (blur material) |
| `border/selection` | `#007AFF` | `#0A84FF` | Active selection rectangle |
| `border/subtle` | `rgba(0,0,0,0.10)` | `rgba(255,255,255,0.10)` | Dividers, borders |
| `text/primary` | `#1D1D1F` | `#F5F5F7` | Primary labels |
| `text/secondary` | `#86868B` | `#A1A1A6` | Dimension labels, hints |
| `accent` | `#007AFF` | `#0A84FF` | Selection, active states |
| `destructive` | `#FF3B30` | `#FF453A` | Cancel, delete |
| `success` | `#34C759` | `#30D158` | Confirm, done |

#### Annotation Colors (user-selectable palette)

| Swatch | Hex | Name |
|--------|-----|------|
| 1 | `#FF3B30` | Red (default) |
| 2 | `#FF9500` | Orange |
| 3 | `#FFCC00` | Yellow |
| 4 | `#34C759` | Green |
| 5 | `#007AFF` | Blue |
| 6 | `#AF52DE` | Purple |
| 7 | `#1D1D1F` | Black |
| 8 | `#FFFFFF` | White |

Plus a custom color picker (macOS system color panel).

### 3.2 Typography

Use **SF Pro** (macOS system font) across all UI.

| Style | Font | Size | Weight | Line Height | Usage |
|-------|------|------|--------|-------------|-------|
| `title` | SF Pro Display | 16px | Semibold | 20px | Settings window title |
| `toolbar-label` | SF Pro Text | 11px | Medium | 14px | Tooltip labels |
| `dimension` | SF Pro Mono | 11px | Regular | 14px | Size indicators (W × H) |
| `menu-item` | SF Pro Text | 13px | Regular | 18px | Menu bar dropdown items |
| `menu-shortcut` | SF Pro Text | 13px | Regular | 18px | Keyboard shortcut labels |
| `annotation-text` | SF Pro Text | 14px | Regular | 18px | Default text annotation |
| `body` | SF Pro Text | 13px | Regular | 18px | Settings body text |

### 3.3 Spacing & Layout Tokens

| Token | Value | Usage |
|-------|-------|-------|
| `space-xs` | 4px | Icon-to-label gap inside toolbar buttons |
| `space-sm` | 8px | Between toolbar items, padding inside pills |
| `space-md` | 12px | Toolbar padding, section spacing |
| `space-lg` | 16px | Between toolbar groups |
| `space-xl` | 24px | Settings section spacing |
| `radius-sm` | 4px | Small buttons, input fields |
| `radius-md` | 8px | Toolbar container, popover |
| `radius-lg` | 12px | Settings window, panels |
| `radius-pill` | 9999px | Color swatches, pill buttons |

### 3.4 Elevation & Materials

| Level | Shadow | Blur | Usage |
|-------|--------|------|-------|
| `elevation-toolbar` | `0 2px 12px rgba(0,0,0,0.15)` | Background blur 20px | Floating annotation toolbar |
| `elevation-popover` | `0 4px 20px rgba(0,0,0,0.20)` | Background blur 30px | Color picker, tool options |
| `elevation-menu` | `0 8px 32px rgba(0,0,0,0.25)` | Background blur 40px | Menu bar dropdown |
| `elevation-pin` | `0 4px 16px rgba(0,0,0,0.30)` | — | Pinned floating image |

### 3.5 Iconography

Use **SF Symbols** where available, custom icons for annotation tools:

| Icon | SF Symbol / Custom | Context |
|------|-------------------|---------|
| Area capture | `viewfinder` | Menu bar |
| Screen capture | `display` | Menu bar |
| Window capture | `macwindow` | Menu bar |
| Scrolling capture | `arrow.up.and.down.text.horizontal` | Menu bar |
| OCR | `text.viewfinder` | Menu bar |
| Rectangle tool | Custom (rounded rect outline) | Toolbar |
| Ellipse tool | Custom (ellipse outline) | Toolbar |
| Arrow tool | Custom (arrow shape) | Toolbar |
| Line tool | Custom (diagonal line) | Toolbar |
| Brush/Pen tool | Custom (pen nib) | Toolbar |
| Text tool | `textformat` or `A` | Toolbar |
| Blur tool | Custom (grid/mosaic) | Toolbar |
| Step counter | Custom (circled "1") | Toolbar |
| Highlight | Custom (marker) | Toolbar |
| Undo | `arrow.uturn.backward` | Toolbar |
| Redo | `arrow.uturn.forward` | Toolbar |
| Save | `square.and.arrow.down` | Toolbar |
| Copy | `doc.on.doc` | Toolbar |
| Close/Cancel | `xmark` | Toolbar |
| Confirm/Done | `checkmark` | Toolbar |
| Pin | `pin` | Toolbar |
| Settings | `gearshape` | Menu bar |

---

## 4. Screen Inventory & User Flows

### 4.1 Core User Flow

```
┌──────────────┐     ┌──────────────────┐     ┌──────────────────┐     ┌──────────────┐
│  Menu Bar    │────▶│  Region Select   │────▶│  Annotation      │────▶│  Output      │
│  (or Hotkey) │     │  Overlay         │     │  Editor          │     │  (Clipboard/ │
│              │     │                  │     │                  │     │   File/Pin)  │
└──────────────┘     └──────────────────┘     └──────────────────┘     └──────────────┘
       │                                             ▲
       ├─── Window Capture ──────────────────────────┤
       ├─── Full Screen ─────────────────────────────┤
       └─── Scrolling Capture ───────────────────────┘
```

### 4.2 Complete Screen Inventory

| # | Screen | States | Frame Count |
|---|--------|--------|-------------|
| 1 | Menu bar dropdown | Default, "more" submenu expanded | 2 |
| 2 | Region selection overlay | Idle (crosshair), Dragging (rubber-band + dimensions), Captured (blue border) | 3 |
| 3 | Window capture overlay | Hovering (window highlight), Captured | 2 |
| 4 | Annotation editor — empty | Just captured, toolbar visible, no annotations yet | 1 |
| 5 | Annotation editor — annotating | With sample annotations (arrow, text, blur, step markers) | 1 |
| 6 | Toolbar — default | All tools in default state | 1 |
| 7 | Toolbar — each tool active | Arrow selected, Rectangle selected, Text selected, etc. | 10 |
| 8 | Tool options popover | Stroke width slider, fill toggle, corner radius, font size | 4 |
| 9 | Color palette popover | 8 swatches + custom color button | 1 |
| 10 | Text editing mode | Inline text cursor on canvas, text input active | 1 |
| 11 | Object selection | Annotation selected, showing handles + bounding box | 1 |
| 12 | Scrolling capture — setup | Region selected, "Start scrolling" button visible | 1 |
| 13 | Scrolling capture — in progress | Preview strip growing, status indicator | 1 |
| 14 | Scrolling capture — complete | Full stitched result in editor | 1 |
| 15 | Pin image — floating window | With close + opacity controls, multiple stacked | 2 |
| 16 | Settings window | General, Shortcuts, Output tabs | 3 |
| | | **Total estimated frames** | **~35** |

### 4.3 Figma Flow Diagram

Create a dedicated **User Flows** page in Figma with:
- A flow diagram using Figma's built-in flow connectors (or FigJam)
- Each screen represented as a labeled thumbnail
- Arrows showing navigation triggers (click, hotkey, drag completion)
- Branching paths for different capture modes

---

## 5. Page 1: Menu Bar & Trigger

### 5.1 Menu Bar Icon

- **Location**: macOS menu bar (right side, among system icons)
- **Icon**: SnapInk logo — a minimal ink drop or pen nib silhouette, 18×18 pt
- **States**: Default (monochrome), Active/Recording (accent color pulse)

### 5.2 Dropdown Menu

Reference: `Screenshot 2026-03-14 at 15.47.33.png` (Shottr menu)

**Layout** — vertical menu with grouped sections, separated by dividers:

```
┌─────────────────────────────────────┐
│  SnapInk                            │
├─────────────────────────────────────┤
│  📷  Capture Screen         ⌘⇧ 3   │
│  ✂️  Capture Area           ⌘⇧ 4   │
│  🪟  Capture Window         ⌘⇧ 5   │
│  📜  Scrolling Capture      ⌘⇧ 6   │
├─────────────────────────────────────┤
│  🔤  Recognize Text (OCR)  ⌘⇧ 7   │
├─────────────────────────────────────┤
│  •••  More                     ▶   │
├─────────────────────────────────────┤
│  ⚙️  Settings...           ⌘ ,     │
│      Quit SnapInk           ⌘ Q     │
└─────────────────────────────────────┘
```

**"More" submenu** (expands on hover):

```
┌──────────────────────────────────┐
│  Repeat Last Capture      ⌘⇧ R  │
│  Capture Active Window    ⌘⇧ 5  │
│  Delayed Screenshot (3s)         │
│  Scrolling (Up)                  │
├──────────────────────────────────┤
│  Open File...                    │
│  Load from Clipboard             │
└──────────────────────────────────┘
```

### 5.3 Figma Specifications

- **Frame size**: Menu width ~220px (main), ~200px (submenu)
- **Row height**: 28px per menu item
- **Typography**: `menu-item` (13px SF Pro Text) for labels, `menu-shortcut` right-aligned
- **Icon size**: 16×16 pt SF Symbols, left-aligned with 12px padding
- **Divider**: 1px line, `border/subtle` color, with 4px vertical margin
- **Background**: `bg/toolbar` with `elevation-menu` shadow + background blur
- **Corner radius**: `radius-md` (8px)
- **Hover state**: Row background `accent` color at 15% opacity
- **Build as**: Figma component with interactive hover variants

---

## 6. Page 2: Region Selection Overlay

### 6.1 Overview

When the user triggers "Capture Area", the entire screen dims and a crosshair cursor appears. The user drags to select a rectangular region.

Reference: `annotation_screeshot_gif.gif` (Xnip area selection flow)

### 6.2 States

#### State 1: Idle (Crosshair)

- **Background**: Full-screen dim overlay (`bg/overlay`)
- **Cursor**: Crosshair (thin lines spanning full screen width/height, centered on cursor)
- **Coordinate display**: Small pill near cursor showing `x, y` position in pixels
  - Background: `rgba(0,0,0,0.75)`, text: white `dimension` style
  - Position: 12px offset from cursor, avoids screen edges

#### State 2: Dragging (Rubber-Band Selection)

- **Selected region**: Clear/transparent (shows screen content through the overlay)
- **Border**: 1px `border/selection` (blue) dashed line around selection
- **Dimension label**: Pill showing `W × H` (e.g., "640 × 480")
  - Positioned centered below the selection, 8px gap
  - Same styling as coordinate pill
- **Corner/Edge handles**: Not visible during initial drag (appears after release)
- **Overlay**: Everything outside the selection remains dimmed

#### State 3: Selection Complete (Pre-Capture)

- **Border**: 1px solid `border/selection` (blue)
- **Resize handles**: 8 handles (4 corners + 4 edge midpoints), 6×6 white squares with 1px blue border
- **Dimension label**: Still visible, updates on resize
- **Action bar**: Compact toolbar appears below selection (or above if near bottom edge):

```
┌─────────────────────────────────────────────────────────────────┐
│  [✂️ Capture]  [📜 Scroll]  [📌 Pin]  │  [✕ Cancel]  [✓ Done]  │
└─────────────────────────────────────────────────────────────────┘
```

### 6.3 Figma Specifications

- **Frame**: 1440×900 (MacBook Pro viewport) or 1920×1080
- **Overlay**: Rectangle fill `bg/overlay`, covering entire frame
- **Selection**: Rectangle with no fill (transparent), clipped from overlay using mask or boolean subtract
- **Crosshair**: Two 1px lines (horizontal + vertical), color `rgba(255,255,255,0.6)`
- **Magnifier/Loupe** (optional detail): Circular 80×80 magnifier near cursor showing 8× zoomed pixels, with border `rgba(255,255,255,0.3)`, positioned offset from cursor

### 6.4 Interaction Notes for Prototype

- Frame 1 (Idle) → on click/drag → Frame 2 (Dragging)
- Frame 2 (Dragging) → on release → Frame 3 (Selection Complete)
- Frame 3 → "Done" button → Annotation Editor
- Frame 3 → "Cancel" or Esc → dismiss overlay

---

## 7. Page 3: Window Capture

### 7.1 Overview

When the user triggers "Capture Window", all windows are detected and highlighted on hover. Clicking captures the highlighted window.

Reference: `window_capture_gif.gif` (Xnip window capture)

### 7.2 States

#### State 1: Window Detection (Hover)

- **Background**: Subtle full-screen dim overlay (`rgba(0,0,0,0.15)`) — lighter than area capture
- **Hovered window**: Bright blue border (3px `border/selection`) around the detected window bounds
- **Window label**: Small pill above the highlighted window showing app name: "Chrome — apple.com"
- **Non-hovered windows**: Slightly dimmed but still visible

#### State 2: Window Captured

- Transition directly to Annotation Editor with the captured window image
- Optional: Brief blue flash/pulse animation on the captured window before transitioning

### 7.3 Multi-Window Capture (Shift+Click)

- After first window captured, overlay remains active
- User holds Shift and clicks additional windows
- Each selected window gets a numbered badge (1, 2, 3...) and a green checkmark
- Final composite combines all selected windows

### 7.4 Shadow Options

- Captured window includes macOS shadow by default
- Option to exclude shadow (toggleable in Settings or via modifier key: Option+Click = no shadow)

### 7.5 Figma Specifications

- **Frame**: Full desktop mockup with 2–3 overlapping windows
- **Highlight border**: 3px stroke, `border/selection`, `radius-md` following window corners
- **Window label pill**: Same styling as dimension pills in Region Selection
- Build the highlight as a Figma component variant: `Window-Highlight/Default`, `Window-Highlight/Selected`

---

## 8. Page 4: Annotation Editor

This is the **primary screen** of the application — where users annotate their captures. Most design effort should be concentrated here.

Reference: `SCR-20260314-nziq.png` (Xnip toolbar), `SCR-20260314-nwkj.png` and `SCR-20260314-nvbz.png` (Shottr editor with annotations)

### 8.1 Layout Structure

```
┌─────────────────────────────────────────────────────────────────────┐
│                                                                     │
│                                                                     │
│                     [ Captured Screenshot ]                         │
│                     (with annotations overlaid)                     │
│                                                                     │
│                                                                     │
│                                                                     │
├─────────────────────────────────────────────────────────────────────┤
│            ┌──── Annotation Toolbar (floating) ────┐                │
│            │  □ ○ ╱ ↗ ✎ ▦ A ① │ ↺ ↻ │ 💾 📋 📌 │ ✕ ✓│             │
│            └───────────────────────────────────────┘                │
│                    ┌── Color Palette ──┐                             │
│                    │ ● ● ● ● ● ● ● ● │                             │
│                    └──────────────────┘                             │
└─────────────────────────────────────────────────────────────────────┘
```

### 8.2 Canvas Area

- **Background**: Transparent checkerboard pattern (or `bg/primary` if opaque)
- **Screenshot**: Centered in the available space with padding
- **Zoom**: Fit-to-view by default, scrollwheel/pinch to zoom, Cmd+0 to reset
- **Pan**: Space+drag or two-finger scroll when zoomed in
- **Canvas border**: Subtle 1px `border/subtle` around the screenshot image

### 8.3 Annotation Toolbar (Primary Component)

The toolbar is a **floating bar** positioned below the captured image, horizontally centered.

#### Toolbar Layout

The toolbar is divided into **4 groups** separated by thin vertical dividers:

```
┌──────────────────────────────────────────────────────────────────────────┐
│ Group 1: Shape Tools │ Group 2: Draw Tools │ Group 3: History │ Group 4: Actions │
│ □  ○  ╱  ↗         │  ✎  ▦  A  ①        │  ↺  ↻           │  💾  📋  ✕  ✓    │
└──────────────────────────────────────────────────────────────────────────┘
```

**Group 1 — Shape Tools**:
| Icon | Tool | Shortcut |
|------|------|----------|
| □ | Rectangle | `R` |
| ○ | Ellipse/Oval | `O` |
| ╱ | Line | `L` |
| ↗ | Arrow | `A` |

**Group 2 — Draw/Content Tools**:
| Icon | Tool | Shortcut |
|------|------|----------|
| ✎ | Freehand/Pen | `P` |
| ▦ | Blur/Pixelate | `B` |
| A | Text | `T` |
| ① | Step Counter | `N` |

**Group 3 — History**:
| Icon | Tool | Shortcut |
|------|------|----------|
| ↺ | Undo | `⌘Z` |
| ↻ | Redo | `⌘⇧Z` |

**Group 4 — Actions**:
| Icon | Tool | Shortcut |
|------|------|----------|
| 💾 | Save to file | `⌘S` |
| 📋 | Copy to clipboard | `⌘C` |
| 📜 | Scrolling Capture | — |
| ✕ | Cancel / Discard | `Esc` |
| ✓ | Confirm / Done | `Enter` |

### 8.4 Toolbar Specifications

- **Container**: Rounded rectangle, `bg/toolbar` with backdrop blur, `elevation-toolbar` shadow
- **Corner radius**: `radius-md` (8px)
- **Height**: 40px
- **Padding**: 8px horizontal, 4px vertical
- **Tool button size**: 28×28 pt hit area, 20×20 pt icon
- **Button states**:
  - **Default**: Icon color `text/primary`, no background
  - **Hover**: Background `rgba(0,0,0,0.06)` (light) / `rgba(255,255,255,0.08)` (dark), `radius-sm`
  - **Active/Selected**: Background `accent` at 15% opacity, icon tinted `accent`
  - **Disabled**: Icon at 30% opacity (e.g., Undo when no history)
- **Group divider**: 1px vertical line, 16px tall, `border/subtle`, 8px horizontal margin
- **Build as**: Component set with variants for each tool's selected state

### 8.5 Color Palette Bar

Positioned directly below the toolbar, centered:

- **Layout**: Horizontal row of 8 circular swatches + 1 custom color button
- **Swatch size**: 16×16 circle (20×20 hit area)
- **Spacing**: 6px between swatches
- **Active indicator**: 2px white ring + 2px gap around selected swatch (ring effect)
- **Custom color button**: Small "+" or rainbow gradient circle at the end, opens macOS system color panel
- **Container**: Same blur material as toolbar, smaller pill shape, `elevation-toolbar`

### 8.6 Tool Options Popover

When a tool is selected and the user needs to adjust settings, a popover appears above the selected tool button.

**Content varies by tool**:

| Tool | Options |
|------|---------|
| Rectangle | Stroke width (1–8px slider), Fill toggle, Corner radius (0–16px slider) |
| Ellipse | Stroke width, Fill toggle |
| Arrow | Stroke width, Head style (filled/open) |
| Line | Stroke width, Dash style (solid/dashed/dotted) |
| Pen/Brush | Stroke width (1–16px), Smoothing toggle |
| Blur | Blur strength (low/medium/high), Mode (blur/pixelate) |
| Text | Font size (12–48px), Bold/Italic toggles |
| Step Counter | Starting number, Size (small/medium/large) |

**Popover specs**:
- **Width**: ~180px (varies by content)
- **Background**: `bg/toolbar` with backdrop blur, `elevation-popover`
- **Corner radius**: `radius-md`
- **Padding**: `space-md`
- **Arrow indicator**: 8px triangle pointing down toward the toolbar button
- **Trigger**: Right-click on tool button, or long-press, or dedicated "..." button

### 8.7 Object Selection State

When an existing annotation is clicked (Pointer/Select mode):

- **Bounding box**: 1px dashed `border/selection` around the object
- **Resize handles**: 8 white squares (6×6px) with 1px `accent` border at corners + edge midpoints
- **Rotation handle**: Small circular handle above the bounding box, connected by a thin line (for shapes that support rotation)
- **Contextual actions**: Small floating bar appears near the object:
  ```
  [🎨 Color] [📋 Duplicate] [🗑 Delete]
  ```

---

## 9. Page 5: Annotation Tools — Detail Specs

Design each tool's **interaction state sequence** as separate frames.

### 9.1 Arrow Tool

**States**:
1. **Cursor**: Crosshair
2. **Drawing**: Thin line from start to cursor, arrowhead preview at cursor end
3. **Completed**: Full arrow with styled head, selected state with handles at both endpoints
4. **Shift held**: Line snaps to nearest 45° angle, visual guide line appears

**Arrowhead rendering**:
- Filled triangle, proportional to stroke width (head size ≈ 3× stroke width)
- Direction follows line angle

### 9.2 Rectangle Tool

**States**:
1. **Cursor**: Crosshair
2. **Drawing**: Rectangle outline growing from start point
3. **Completed**: Rectangle with stroke and optional fill
4. **Shift held**: Constrains to square

**Options**: Stroke width, fill color (or no fill), corner radius

### 9.3 Ellipse Tool

**States**: Same as Rectangle but elliptical shape
- **Shift held**: Constrains to circle

### 9.4 Text Tool

**States**:
1. **Cursor**: I-beam text cursor
2. **Click**: Places blinking cursor on canvas, shows inline text input area
3. **Typing**: Text appears in real-time at the clicked position, with a subtle background highlight for readability
4. **Completed**: Text rendered as an annotation object, selectable

**Inline editor specs**:
- No visible text box border during editing (appears on selection only)
- Blinking cursor (1px `accent` color, 500ms blink)
- Text color follows current annotation color
- Font size adjustable via tool options
- Multi-line support: Enter for newline (Cmd+Enter to commit)

### 9.5 Blur/Pixelate Tool

**States**:
1. **Cursor**: Crosshair with small mosaic icon badge
2. **Drawing**: Dashed rectangle showing blur region preview
3. **Completed**: Region shows blurred/pixelated content from underlying image

**Design the blurred region** in Figma:
- Use Figma's `Layer Blur` effect on a clipped copy of the screenshot region
- Or use a mosaic overlay pattern for pixelation preview

### 9.6 Step Counter Tool

**States**:
1. **Cursor**: Crosshair with small "1" badge (or next number)
2. **Click**: Places a numbered circle at the click position
3. **Sequential**: Next click places the next number (2, 3, 4...)

**Badge specs**:
- Circle: 24×24px, filled with current annotation color
- Number: White, SF Pro Bold, 13px, centered
- Auto-increment within session
- Can manually edit number by double-clicking

### 9.7 Freehand/Pen Tool

**States**:
1. **Cursor**: Pen tip icon
2. **Drawing**: Path appears following cursor movement
3. **Completed**: Smoothed path as annotation object

**Smoothing**: Slight Catmull-Rom interpolation to smooth hand jitter

### 9.8 Highlight/Marker Tool

**States**:
1. **Cursor**: Marker tip icon
2. **Drawing**: Semi-transparent colored rectangle or freehand stroke
3. **Completed**: Highlight with multiply blend mode

**Specs**: 50% opacity, 20px default width, color follows palette

---

## 10. Page 6: Scrolling Capture

Reference: `SCR-20260314-nztd.png` (Xnip scrolling capture documentation), `scrolling_catpure_gif.gif`

### 10.1 Flow

```
Region Selection → Click "Scroll" button → Scrolling Mode → Auto-scroll + capture → Stitched result → Editor
```

### 10.2 States

#### State 1: Region Selected with Scroll Button

- After area selection, if scrollable content is detected, show a prominent "Scrolling Capture" button in the action bar
- Button icon: `arrow.up.and.down.text.horizontal`
- Instruction text: "Select the scrollable content area. Floating views should not be included."

#### State 2: Scrolling In Progress

- **Status indicator**: Floating pill at top of selection showing:
  ```
  ┌────────────────────────────┐
  │ 📜 Capturing...  3 frames  │
  └────────────────────────────┘
  ```
- **Preview strip**: Thin vertical preview of the growing stitched image, shown beside the capture area
- **Progress bar**: Subtle progress indicator showing approximate completion
- **Cancel button**: Small "✕" to abort

#### State 3: Scrolling Complete

- Transition to Annotation Editor with the full stitched long image
- Editor canvas scrolls vertically to accommodate the tall image
- Toolbar position: Fixed at bottom of viewport (not scrolling with canvas)

### 10.3 Figma Specifications

- Design State 1 as an extension of the Region Selection Complete frame
- State 2: Show an animation-suggestive mockup with a growing preview strip
- State 3: Tall frame (e.g., 1440×3000) showing a long stitched screenshot in the editor

---

## 11. Page 7: Pin Image (Floating Window)

Reference: `SCR-20260314-oahr.png` (Xnip pin images)

### 11.1 Overview

Pin creates a borderless, always-on-top floating window displaying the screenshot. Multiple pins can coexist on screen.

### 11.2 Pin Window Design

```
┌─────────────────────────────────────────────┐
│  ╳                              🔵          │  ← Close (red) + resize (blue) buttons
│                                              │     visible on hover only
│                                              │
│           [ Pinned Screenshot ]              │
│                                              │
│                                              │
├─────────────────────────────────────────────┤
│                  100%                        │  ← Opacity/zoom controls on hover
└─────────────────────────────────────────────┘
```

### 11.3 States

| State | Behavior |
|-------|----------|
| **Default** | Image only, no chrome. Subtle 1px border + `elevation-pin` shadow. |
| **Hover** | Close button (red ✕, top-left) and resize handle (bottom-right) appear with fade-in. Opacity slider appears at bottom. |
| **Dragging** | Window follows cursor with slight scale-down (0.97×) effect. |
| **Resized** | Image scales proportionally. Minimum: 100×100px. |

### 11.4 Controls (visible on hover)

- **Close**: Red circle with ✕ (macOS traffic light style), top-left corner, 12×12px
- **Resize**: Blue circle handle at bottom-right corner
- **Opacity slider**: Small pill at bottom center: "100%" label + drag to adjust 20%–100%

### 11.5 Multiple Pins

- Design a frame showing 2–3 pinned images at different positions on the desktop
- Slightly cascaded to show depth/stacking
- Each pin has independent controls

### 11.6 Figma Specifications

- **Frame**: Desktop mockup (1440×900) with 2–3 pinned images floating over other content
- **Pin shadow**: `elevation-pin`
- **Border radius**: `radius-sm` (4px)
- **Close/resize buttons**: Build as component variants (hidden/visible)

---

## 12. Page 8: Settings / Preferences

### 12.1 Window Design

Standard macOS preferences window with a tabbed toolbar.

**Size**: 520×400px (fixed, non-resizable)

### 12.2 Tab Structure

#### Tab 1: General

```
┌─ General ─────────────────────────────────────────────┐
│                                                        │
│  Startup                                               │
│  ┌──────────────────────────────────────────────────┐  │
│  │ ☑ Launch at Login                                │  │
│  │ ☑ Show menu bar icon                             │  │
│  │ ☐ Play sound on capture                          │  │
│  └──────────────────────────────────────────────────┘  │
│                                                        │
│  After Capture                                         │
│  ┌──────────────────────────────────────────────────┐  │
│  │ ◉ Open in annotation editor                      │  │
│  │ ○ Copy to clipboard immediately                  │  │
│  │ ○ Save to file immediately                       │  │
│  │                                                    │ │
│  │ ☑ Also copy to clipboard after save               │  │
│  └──────────────────────────────────────────────────┘  │
│                                                        │
│  Appearance                                            │
│  ┌──────────────────────────────────────────────────┐  │
│  │ Theme:  [System ▼]  (System / Light / Dark)      │  │
│  └──────────────────────────────────────────────────┘  │
└────────────────────────────────────────────────────────┘
```

#### Tab 2: Shortcuts

```
┌─ Shortcuts ───────────────────────────────────────────┐
│                                                        │
│  Capture Shortcuts                                     │
│  ┌──────────────────────────────────────────────────┐  │
│  │  Capture Area        [ ⌘ ⇧ 4 ]  [Record]       │  │
│  │  Capture Screen      [ ⌘ ⇧ 3 ]  [Record]       │  │
│  │  Capture Window      [ ⌘ ⇧ 5 ]  [Record]       │  │
│  │  Scrolling Capture   [ ⌘ ⇧ 6 ]  [Record]       │  │
│  │  Recognize Text      [ ⌘ ⇧ 7 ]  [Record]       │  │
│  │  Repeat Last         [ ⌘ ⇧ R ]  [Record]       │  │
│  └──────────────────────────────────────────────────┘  │
│                                                        │
│  ⓘ Click "Record" and press your desired shortcut     │
│    to change the binding.                              │
└────────────────────────────────────────────────────────┘
```

#### Tab 3: Output

```
┌─ Output ──────────────────────────────────────────────┐
│                                                        │
│  Save Location                                         │
│  ┌──────────────────────────────────────────────────┐  │
│  │  📁 ~/Desktop/Screenshots      [Change...]      │  │
│  └──────────────────────────────────────────────────┘  │
│                                                        │
│  Filename Pattern                                      │
│  ┌──────────────────────────────────────────────────┐  │
│  │  [ SnapInk {YYYY-MM-DD} at {HH.mm.ss}      ]    │  │
│  │  Preview: SnapInk 2026-03-14 at 15.47.33.png    │  │
│  └──────────────────────────────────────────────────┘  │
│                                                        │
│  Format                                                │
│  ┌──────────────────────────────────────────────────┐  │
│  │  Format:  [PNG ▼]  (PNG / JPEG / WebP)           │  │
│  │  JPEG Quality: [═══════●══] 85%                  │  │
│  └──────────────────────────────────────────────────┘  │
│                                                        │
│  Clipboard                                             │
│  ┌──────────────────────────────────────────────────┐  │
│  │ ☑ Copy at 2× (Retina) resolution                │  │
│  └──────────────────────────────────────────────────┘  │
└────────────────────────────────────────────────────────┘
```

### 12.3 Figma Specifications

- **Window style**: macOS NSWindow with toolbar tabs, traffic light buttons
- **Tabs**: Segmented control or macOS-style toolbar icons at top
- **Form controls**: Use macOS-native checkbox, radio button, dropdown, slider, text field styles
- **Build each tab as a variant** of a Settings component

---

## 13. Interactive Prototype Wiring

### 13.1 Prototype Flow Map

Wire the following interactions in Figma's Prototype mode:

```
Menu Bar Dropdown
├── "Capture Area"    → Region Selection (Idle)
├── "Capture Screen"  → Annotation Editor (with full-screen image)
├── "Capture Window"  → Window Capture (Hover)
├── "Scrolling"       → Region Selection → Scrolling Mode
├── "Settings..."     → Settings Window
└── "More" hover      → More Submenu

Region Selection (Idle)
└── Drag start        → Region Selection (Dragging)
    └── Drag end      → Region Selection (Complete)
        ├── "Done"    → Annotation Editor
        ├── "Scroll"  → Scrolling Capture (In Progress)
        ├── "Pin"     → Pin Image
        └── "Cancel"  → Dismiss

Window Capture (Hover)
└── Click window      → Annotation Editor (with window image)

Annotation Editor
├── Tool click        → Tool Selected (toolbar state change)
├── Tool right-click  → Tool Options Popover
├── Color swatch      → Color change
├── Canvas click/drag → Create annotation (tool-specific)
├── Annotation click  → Object Selection state
├── Undo              → Previous state
├── Save              → Save dialog / auto-save
├── Copy              → Clipboard (brief toast "Copied!")
├── Pin               → Pin Image (floating window)
├── ✕ Cancel          → Discard confirmation → Dismiss
└── ✓ Done            → Save + dismiss

Scrolling Capture
├── In Progress       → (auto-advance animation)
└── Complete          → Annotation Editor (tall image)

Pin Image
├── Hover             → Show controls
├── ✕ Close           → Dismiss pin
└── Drag              → Reposition
```

### 13.2 Transition Animations

| Transition | Animation | Duration | Easing |
|------------|-----------|----------|--------|
| Menu open | Fade in + scale from 0.97 | 150ms | ease-out |
| Region overlay appear | Fade in | 200ms | ease-out |
| Selection → Editor | Dissolve / Smart animate | 300ms | ease-in-out |
| Tool switch | Instant (no animation) | 0ms | — |
| Tool options popover | Fade in + slide up 4px | 150ms | ease-out |
| Color change | Instant | 0ms | — |
| Toast notification | Slide up + fade in, auto-dismiss after 2s | 200ms/200ms | ease-out |
| Pin window appear | Scale from 0.9 + fade in | 250ms | spring |
| Settings window | macOS standard open (scale from center) | 250ms | ease-out |

### 13.3 Prototype Settings

- **Device**: MacBook Pro 14" (or "None" for custom size)
- **Frame size**: 1440×900 for main flows, 1440×3000 for scrolling capture result
- **Background**: Desktop wallpaper image (use a macOS Sonoma/Sequoia default wallpaper as base)
- **Starting frame**: Menu Bar Dropdown (Default)

---

## 14. Component Library Checklist

Build these as reusable **Figma components** before composing screens:

### 14.1 Atoms

- [ ] Color swatch (8 annotation colors + custom, with active ring)
- [ ] Tool button (icon + label variant, states: default/hover/active/disabled)
- [ ] Menu item (icon + label + shortcut, states: default/hover)
- [ ] Dimension pill (e.g., "640 × 480", dark translucent background)
- [ ] Resize handle (6×6 white square with blue border)
- [ ] Selection bounding box (dashed blue rect + 8 handles)
- [ ] macOS traffic light buttons (close/minimize/zoom)
- [ ] Checkbox (macOS native style)
- [ ] Radio button (macOS native style)
- [ ] Dropdown select (macOS native style)
- [ ] Slider (macOS native style)
- [ ] Text input field (macOS native style)
- [ ] Toast notification ("Copied to clipboard!", etc.)
- [ ] Keyboard shortcut badge (e.g., "⌘⇧4")

### 14.2 Molecules

- [ ] Annotation toolbar (Groups 1–4 assembled, with all tool variants)
- [ ] Color palette bar (8 swatches + custom)
- [ ] Tool options popover (per-tool variants)
- [ ] Menu bar dropdown (with all items + states)
- [ ] "More" submenu
- [ ] Selection action bar (Capture / Scroll / Pin / Cancel / Done)
- [ ] Scrolling capture status indicator
- [ ] Object contextual action bar (Color / Duplicate / Delete)
- [ ] Settings tab bar
- [ ] Shortcut recorder row

### 14.3 Organisms

- [ ] Region Selection Overlay (complete with crosshair + selection + toolbar)
- [ ] Window Capture Overlay (with window highlight)
- [ ] Annotation Editor (canvas + toolbar + palette)
- [ ] Pin Image Window (with hover controls)
- [ ] Settings Window (with all 3 tabs)

### 14.4 Annotation Samples (for mockups)

- [ ] Red arrow annotation (pointing at something)
- [ ] Blue rectangle highlight
- [ ] Text annotation ("buy this!")
- [ ] Blurred/pixelated region
- [ ] Step counter badges (1, 2, 3)
- [ ] Freehand circle
- [ ] Highlight marker stroke

---

## 15. Design Deliverables & Handoff

### 15.1 Deliverable List

| # | Deliverable | Format | Purpose |
|---|-------------|--------|---------|
| 1 | **Figma Design File** | Figma | Source of truth for all screens and components |
| 2 | **Interactive Prototype** | Figma Prototype | Clickable walkthrough of all user flows |
| 3 | **Component Library** | Figma Components (in Design System page) | Reusable components for development |
| 4 | **Icon Set** | SVG export from Figma | All custom annotation tool icons |
| 5 | **Color & Typography Tokens** | Figma Variables (exportable as JSON) | Design tokens for code |
| 6 | **Screen Inventory Spreadsheet** | Figma section or linked doc | All frames with status (draft/review/final) |
| 7 | **Annotation Spec** | Figma Dev Mode inspect | Spacing, sizing, color values per screen |

### 15.2 Developer Handoff Notes

When handing off to development, ensure:

- All frames have **Auto Layout** for responsive behavior inspection
- Colors, spacing, and radii use **Figma Variables** (not hardcoded values)
- Components have **clear variant names** that map to code states
- Interactive prototype covers **all primary flows** for reference
- Export all custom icons as **SVG** (outline strokes, unified viewbox)
- Document any **animation specs** (duration, easing, properties) in a text layer or comment

---

## 16. Implementation Phases

### Phase 1: Design System & Core Components (Week 1)

**Goal**: Establish the design foundation.

- [ ] Set up Figma file with page structure
- [ ] Define all color variables (light + dark mode)
- [ ] Define typography styles
- [ ] Build all atom components (buttons, inputs, swatches, handles)
- [ ] Build annotation toolbar component with all tool variants
- [ ] Build color palette bar component
- [ ] Build menu bar dropdown component
- [ ] Source/create all icons (SF Symbols + custom tool icons)

### Phase 2: Capture Flows (Week 2)

**Goal**: Design all capture-mode screens.

- [ ] Region Selection — 3 states (Idle, Dragging, Complete)
- [ ] Window Capture — 2 states (Hover, Captured)
- [ ] Full-screen capture — transition to editor
- [ ] Menu bar dropdown — all items + "More" submenu
- [ ] Wire basic prototype flow: Menu → Selection → Editor

### Phase 3: Annotation Editor (Week 2–3)

**Goal**: Design the main editing experience — this is the product's core.

- [ ] Editor layout with canvas + toolbar + palette
- [ ] Tool options popovers (for each tool)
- [ ] Object selection state (bounding box, handles, contextual actions)
- [ ] Text editing inline state
- [ ] Sample annotation mockups showing real-world usage
- [ ] Blur/pixelate visual treatment
- [ ] Step counter placement mockup

### Phase 4: Advanced Features (Week 3)

**Goal**: Design scrolling capture, pin, and settings.

- [ ] Scrolling capture — 3 states (Setup, In Progress, Complete)
- [ ] Pin image — floating window with hover controls
- [ ] Multiple pins on desktop mockup
- [ ] Settings window — 3 tabs (General, Shortcuts, Output)
- [ ] Shortcut recorder interaction

### Phase 5: Prototype Wiring & Polish (Week 4)

**Goal**: Connect all screens into a fully interactive prototype.

- [ ] Wire all prototype connections (per flow map in Section 13)
- [ ] Add transition animations (per animation table)
- [ ] Create a "demo flow" starting frame optimized for presentation
- [ ] Review all states for completeness (hover, active, disabled, error)
- [ ] Dark mode variants for all key screens (at minimum: Editor + Toolbar)
- [ ] Add annotation comments on complex interactions for developer reference
- [ ] Final review pass: consistency, spacing, alignment, naming

### Phase 6: Handoff Preparation (Week 4)

**Goal**: Prepare deliverables for engineering.

- [ ] Verify all components use Figma Variables
- [ ] Export icon set as SVG
- [ ] Write annotation specs for complex interactions
- [ ] Create screen inventory with status tags
- [ ] Share prototype link with stakeholders for feedback

---

## Appendix A: Reference Image Index

| File | Content | Relevant To |
|------|---------|-------------|
| `SCR-20260314-nvbz.png` | Shottr editor — blur annotation on webpage | Annotation Editor, Blur tool |
| `SCR-20260314-nvsb.png` | Shottr editor — highlight/spotlight annotation | Annotation Editor, Highlight tool |
| `SCR-20260314-nwkj.png` | Shottr editor — area selection with toolbar visible | Region Selection, Toolbar layout |
| `SCR-20260314-nxhr.png` | Shottr editor — pixelate annotation | Blur/Pixelate tool |
| `SCR-20260314-nziq.png` | Xnip — full toolbar with all tools labeled | Toolbar design (primary reference) |
| `SCR-20260314-nztd.png` | Xnip — scrolling capture documentation | Scrolling Capture flow |
| `SCR-20260314-oahr.png` | Xnip — Pin Images feature | Pin Image design |
| `Screenshot 2026-03-14 at 15.47.33.png` | Shottr — menu bar dropdown with all options | Menu Bar design (primary reference) |
| `annotation_screeshot_gif.gif` | Xnip — annotation workflow demo | Annotation Editor interaction flow |
| `window_capture_gif.gif` | Xnip — window capture with hover highlight | Window Capture design |
| `scrolling_catpure_gif.gif` | Xnip — scrolling capture in action | Scrolling Capture flow |

## Appendix B: Keyboard Shortcut Map

| Action | Shortcut | Context |
|--------|----------|---------|
| Capture Area | `⌘⇧4` | Global |
| Capture Screen | `⌘⇧3` | Global |
| Capture Window | `⌘⇧5` | Global |
| Scrolling Capture | `⌘⇧6` | Global |
| OCR / Recognize Text | `⌘⇧7` | Global |
| Repeat Last Capture | `⌘⇧R` | Global |
| Rectangle Tool | `R` | Editor |
| Ellipse Tool | `O` | Editor |
| Line Tool | `L` | Editor |
| Arrow Tool | `A` | Editor |
| Pen/Freehand Tool | `P` | Editor |
| Blur/Pixelate Tool | `B` | Editor |
| Text Tool | `T` | Editor |
| Step Counter Tool | `N` | Editor |
| Pointer/Select | `V` | Editor |
| Undo | `⌘Z` | Editor |
| Redo | `⌘⇧Z` | Editor |
| Copy to Clipboard | `⌘C` | Editor |
| Save to File | `⌘S` | Editor |
| Confirm / Done | `Enter` | Editor |
| Cancel / Discard | `Esc` | Editor / Selection |
| Constrain Proportions | `Shift` (hold) | Drawing |
| Nudge 1px | `Arrow Keys` | Selection |
| Nudge 10px | `Shift + Arrow Keys` | Selection |
| Zoom In | `⌘+` or scroll up | Editor |
| Zoom Out | `⌘-` or scroll down | Editor |
| Zoom to Fit | `⌘0` | Editor |
| Delete Annotation | `Delete` / `Backspace` | Selection |
