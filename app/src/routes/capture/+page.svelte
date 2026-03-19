<script lang="ts">
  import { onMount } from 'svelte';
  import { invoke } from '@tauri-apps/api/core';
  import { getCurrentWindow } from '@tauri-apps/api/window';
  import type { CaptureMode, OverlayState, MonitorInfo, WindowInfo, Point } from '$lib/types';

  type SelectionRect = { x: number; y: number; width: number; height: number };

  let mode = $state<CaptureMode>('area');
  let overlayState = $state<OverlayState>('idle');
  let cursor = $state<Point>({ x: 0, y: 0 });
  let dragStart = $state<Point | null>(null);
  let selection = $state<SelectionRect | null>(null);
  let screenshotData = $state<string | null>(null);
  let monitors = $state<MonitorInfo[]>([]);
  let windows = $state<WindowInfo[]>([]);
  let hoveredWindowId = $state<number | null>(null);

  const appWindow = getCurrentWindow();

  function getUrlMode(): CaptureMode {
    if (typeof window === 'undefined') return 'area';
    const params = new URLSearchParams(window.location.search);
    return (params.get('mode') as CaptureMode) ?? 'area';
  }

  onMount(async () => {
    mode = getUrlMode();
    try {
      monitors = await invoke<MonitorInfo[]>('get_monitors');
      if (mode === 'window') {
        windows = await invoke<WindowInfo[]>('get_windows');
      }
      // Use the pre-taken background screenshot (captured before this window opened)
      screenshotData = await invoke<string | null>('get_capture_background');
    } catch (e) {
      console.error(e);
    }
    // Show the window only after the screenshot background is ready to render.
    // The window starts hidden (visible: false in Rust) to prevent the
    // black/white flash that would occur while the webview is loading.
    await appWindow.show();
    await appWindow.setFocus();
  });


  function onMouseMove(e: MouseEvent) {
    cursor = { x: e.clientX, y: e.clientY };

    if (overlayState === 'dragging' && dragStart) {
      const x = Math.min(dragStart.x, e.clientX);
      const y = Math.min(dragStart.y, e.clientY);
      const width = Math.abs(e.clientX - dragStart.x);
      const height = Math.abs(e.clientY - dragStart.y);
      selection = { x, y, width, height };
    }

    if (mode === 'window') {
      updateHoveredWindow(e.clientX, e.clientY);
    }
  }

  function updateHoveredWindow(mx: number, my: number) {
    const win = windows.find((w) => {
      const scaleFactor = monitors[0]?.scaleFactor ?? 1;
      const wx = w.x / scaleFactor;
      const wy = w.y / scaleFactor;
      const ww = w.width / scaleFactor;
      const wh = w.height / scaleFactor;
      return mx >= wx && mx <= wx + ww && my >= wy && my <= wy + wh;
    });
    hoveredWindowId = win?.id ?? null;
  }

  function onMouseDown(e: MouseEvent) {
    if (e.button !== 0) return;
    if (overlayState === 'idle' && (mode === 'area' || mode === 'scrolling')) {
      dragStart = { x: e.clientX, y: e.clientY };
      overlayState = 'dragging';
      selection = { x: e.clientX, y: e.clientY, width: 0, height: 0 };
    }
    if (mode === 'window' && hoveredWindowId !== null) {
      captureWindow(hoveredWindowId);
    }
  }

  function onMouseUp(e: MouseEvent) {
    if (overlayState === 'dragging') {
      if (selection && selection.width > 4 && selection.height > 4) {
        overlayState = 'complete';
      } else {
        overlayState = 'idle';
        dragStart = null;
        selection = null;
      }
    }
  }

  async function captureWindow(windowId: number) {
    try {
      const data = await invoke<string>('capture_window_by_id', { windowId });
      await invoke('store_capture_result', { data });
      await invoke('open_editor_cmd');
    } catch (e) {
      console.error(e);
    }
    await appWindow.close();
  }

  async function doCapture() {
    if (!selection || !monitors[0]) return;
    const scale = monitors[0].scaleFactor;
    try {
      await invoke('crop_and_store', {
        x: Math.round(selection.x * scale),
        y: Math.round(selection.y * scale),
        width: Math.round(selection.width * scale),
        height: Math.round(selection.height * scale),
      });
      await invoke('open_editor_cmd');
    } catch (e) {
      console.error(e);
    }
    await appWindow.close();
  }

  /// Start a scrolling capture session: store the region in Rust,
  /// close this overlay, and open the small scroll control window.
  async function startScrollCapture() {
    if (!selection || !monitors[0]) return;
    const scale = monitors[0].scaleFactor;
    try {
      await invoke('start_scroll_capture_cmd', {
        x: Math.round(selection.x * scale),
        y: Math.round(selection.y * scale),
        width: Math.round(selection.width * scale),
        height: Math.round(selection.height * scale),
        logicalX: selection.x,
        logicalY: selection.y,
        logicalWidth: selection.width,
        logicalHeight: selection.height,
      });
      // Rust closes this window and opens the control window.
    } catch (e) {
      console.error('startScrollCapture failed:', e);
    }
  }

  async function cancel() {
    await appWindow.close();
  }

  function selectionStyle(sel: SelectionRect) {
    return `left:${sel.x}px;top:${sel.y}px;width:${sel.width}px;height:${sel.height}px`;
  }

  function dimLeft(sel: SelectionRect) {
    return `left:0;top:0;width:${sel.x}px;height:100%`;
  }
  function dimRight(sel: SelectionRect) {
    return `left:${sel.x + sel.width}px;top:0;right:0;height:100%`;
  }
  function dimTop(sel: SelectionRect) {
    return `left:${sel.x}px;top:0;width:${sel.width}px;height:${sel.y}px`;
  }
  function dimBottom(sel: SelectionRect) {
    return `left:${sel.x}px;top:${sel.y + sel.height}px;width:${sel.width}px;bottom:0`;
  }

  function actionBarStyle(sel: SelectionRect): string {
    const barY = sel.y + sel.height + 14;
    const barX = sel.x + sel.width / 2;
    return `left:${barX}px;top:${barY}px;transform:translateX(-50%)`;
  }

  function pillStyle(sel: SelectionRect): string {
    if (overlayState === 'complete') {
      return `left:${sel.x + sel.width / 2}px;top:${sel.y - 32}px;transform:translateX(-50%)`;
    }
    return `left:${sel.x + sel.width / 2}px;top:${sel.y + sel.height + 8}px;transform:translateX(-50%)`;
  }

  function hoveredWindow() {
    return windows.find((w) => w.id === hoveredWindowId);
  }
</script>

<svelte:window onkeydown={(e) => e.key === 'Escape' && cancel()} />

<div
  class="overlay"
  role="presentation"
  onmousemove={onMouseMove}
  onmousedown={onMouseDown}
  onmouseup={onMouseUp}
>
  {#if screenshotData}
    <img class="bg-screenshot" src={`data:image/png;base64,${screenshotData}`} alt="" />
  {/if}

  {#if mode === 'area' || mode === 'scrolling'}
    {#if !selection}
      <!-- Idle: full dim overlay -->
      <div class="dim-full"></div>
    {:else}
      <!-- Dim sides -->
      <div class="dim" style={dimLeft(selection)}></div>
      <div class="dim" style={dimRight(selection)}></div>
      <div class="dim" style={dimTop(selection)}></div>
      <div class="dim" style={dimBottom(selection)}></div>

      <!-- Selection rectangle -->
      <div
        class="selection-rect"
        class:dashed={overlayState === 'dragging'}
        class:solid={overlayState === 'complete'}
        style={selectionStyle(selection)}
      >
        {#if overlayState === 'complete'}
          <div class="handle tl"></div>
          <div class="handle tc"></div>
          <div class="handle tr"></div>
          <div class="handle ml"></div>
          <div class="handle mr"></div>
          <div class="handle bl"></div>
          <div class="handle bc"></div>
          <div class="handle br"></div>
        {/if}
      </div>

      <!-- Dimension pill -->
      <div class="dimension-pill" style={pillStyle(selection)}>
        {Math.round(selection.width)} × {Math.round(selection.height)}
      </div>
    {/if}

    <!-- Crosshair -->
    {#if overlayState === 'idle'}
      <div class="crosshair-h" style={`top:${cursor.y}px`}></div>
      <div class="crosshair-v" style={`left:${cursor.x}px`}></div>
      <div class="cursor-dot" style={`left:${cursor.x}px;top:${cursor.y}px`}></div>
      <div class="coord-pill" style={`left:${cursor.x + 14}px;top:${cursor.y + 14}px`}>
        {cursor.x}, {cursor.y}
      </div>
      {#if mode === 'scrolling'}
        <div class="instruction">Select the scrollable content area</div>
      {:else}
        <div class="instruction">Click and drag to select area</div>
      {/if}
    {/if}

    <!-- Action bar -->
    {#if overlayState === 'complete' && selection}
      {#if mode === 'scrolling'}
        <!-- Scrolling mode: only show Start and Cancel -->
        <div class="action-bar" style={actionBarStyle(selection)}>
          <button class="action-btn scroll-start" onclick={startScrollCapture}>
            ↕ Start Scrolling Capture
          </button>
          <div class="action-divider"></div>
          <button class="action-btn danger" onclick={cancel}>✕</button>
        </div>
      {:else}
        <!-- Area mode: standard capture buttons -->
        <div class="action-bar" style={actionBarStyle(selection)}>
          <button class="action-btn primary" onclick={doCapture}>✂ Capture</button>
          <div class="action-divider"></div>
          <button class="action-btn secondary" onclick={startScrollCapture}>↕ Scroll</button>
          <button class="action-btn secondary">📌 Pin</button>
          <div class="action-divider"></div>
          <button class="action-btn danger" onclick={cancel}>✕</button>
          <button class="action-btn success" onclick={doCapture}>✓</button>
        </div>
      {/if}
    {/if}
  {/if}

  {#if mode === 'window'}
    <div class="dim-light"></div>
    {#if hoveredWindowId !== null}
      {@const win = hoveredWindow()}
      {#if win}
        {@const scale = monitors[0]?.scaleFactor ?? 1}
        {@const wx = win.x / scale}
        {@const wy = win.y / scale}
        {@const ww = win.width / scale}
        {@const wh = win.height / scale}
        <div
          class="window-highlight"
          style={`left:${wx}px;top:${wy}px;width:${ww}px;height:${wh}px`}
        ></div>
        <div class="window-label" style={`left:${wx}px;top:${wy - 36}px`}>
          {win.appName} — {win.title}
        </div>
      {/if}
    {/if}
    <div class="window-instruction">Click to capture • Esc to cancel</div>
  {/if}
</div>

<style>
.overlay {
  position: fixed;
  inset: 0;
  overflow: hidden;
  cursor: crosshair;
  user-select: none;
}

.bg-screenshot {
  position: absolute;
  inset: 0;
  width: 100%;
  height: 100%;
  object-fit: cover;
  pointer-events: none;
}

.dim-full {
  position: absolute;
  inset: 0;
  background: rgba(0, 0, 0, 0.45);
  pointer-events: none;
}

.dim-light {
  position: absolute;
  inset: 0;
  background: rgba(0, 0, 0, 0.15);
  pointer-events: none;
}

.dim {
  position: absolute;
  background: rgba(0, 0, 0, 0.45);
  pointer-events: none;
}

.selection-rect {
  position: absolute;
  background: rgba(0, 122, 255, 0.04);
  pointer-events: none;
}

.selection-rect.dashed {
  border: 1.5px dashed #007AFF;
}

.selection-rect.solid {
  border: 1.5px solid #007AFF;
}

.handle {
  position: absolute;
  width: 8px;
  height: 8px;
  background: white;
  border: 1.5px solid #007AFF;
  border-radius: 1px;
  transform: translate(-50%, -50%);
}

.handle.tl { left: 0; top: 0; }
.handle.tc { left: 50%; top: 0; }
.handle.tr { left: 100%; top: 0; }
.handle.ml { left: 0; top: 50%; }
.handle.mr { left: 100%; top: 50%; }
.handle.bl { left: 0; top: 100%; }
.handle.bc { left: 50%; top: 100%; }
.handle.br { left: 100%; top: 100%; }

.crosshair-h {
  position: absolute;
  left: 0;
  right: 0;
  height: 1px;
  background: rgba(255, 255, 255, 0.55);
  pointer-events: none;
}

.crosshair-v {
  position: absolute;
  top: 0;
  bottom: 0;
  width: 1px;
  background: rgba(255, 255, 255, 0.55);
  pointer-events: none;
}

.cursor-dot {
  position: absolute;
  width: 10px;
  height: 10px;
  background: white;
  border-radius: 50%;
  transform: translate(-50%, -50%);
  pointer-events: none;
}

.coord-pill,
.dimension-pill {
  position: absolute;
  background: rgba(0, 0, 0, 0.72);
  color: white;
  font-family: 'SF Mono', 'Menlo', monospace;
  font-size: 11px;
  padding: 3px 8px;
  border-radius: 9999px;
  white-space: nowrap;
  pointer-events: none;
}

.instruction {
  position: absolute;
  bottom: 24px;
  left: 50%;
  transform: translateX(-50%);
  color: rgba(255, 255, 255, 0.5);
  font-size: 12px;
  pointer-events: none;
}

.action-bar {
  position: absolute;
  display: flex;
  align-items: center;
  gap: 2px;
  background: rgba(255, 255, 255, 0.92);
  backdrop-filter: blur(20px);
  border: 1px solid rgba(0, 0, 0, 0.08);
  border-radius: var(--radius-md);
  padding: 4px 6px;
  box-shadow: var(--elevation-toolbar);
}

.action-btn {
  padding: 4px 10px;
  border-radius: 5px;
  font-size: 12px;
  font-weight: 500;
  color: #1D1D1F;
  cursor: pointer;
  white-space: nowrap;
  background: none;
  border: none;
}

.action-btn:hover {
  background: rgba(0, 0, 0, 0.06);
}

.action-btn.primary {
  background: #007AFF;
  color: white;
}

.action-btn.primary:hover {
  background: #0066CC;
}

.action-btn.scroll-start {
  background: #34C759;
  color: white;
}

.action-btn.scroll-start:hover {
  background: #28A745;
}

.action-btn.danger {
  color: #FF3B30;
}

.action-btn.success {
  color: #34C759;
}

.action-divider {
  width: 1px;
  height: 18px;
  background: rgba(0, 0, 0, 0.1);
  margin: 0 2px;
}

.window-highlight {
  position: absolute;
  border: 3px solid #007AFF;
  border-radius: 10px;
  pointer-events: none;
}

.window-label {
  position: absolute;
  background: rgba(0, 0, 0, 0.72);
  color: white;
  font-size: 11px;
  padding: 3px 10px;
  border-radius: 9999px;
  white-space: nowrap;
  pointer-events: none;
  max-width: 300px;
  overflow: hidden;
  text-overflow: ellipsis;
}

.window-instruction {
  position: absolute;
  bottom: 24px;
  left: 50%;
  transform: translateX(-50%);
  background: rgba(0, 0, 0, 0.65);
  color: white;
  font-size: 12px;
  padding: 6px 14px;
  border-radius: 9999px;
  white-space: nowrap;
  pointer-events: none;
}
</style>
