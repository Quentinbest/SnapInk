<script lang="ts">
  import { onMount, onDestroy, tick } from 'svelte';
  import { invoke } from '@tauri-apps/api/core';
  import { listen, type UnlistenFn } from '@tauri-apps/api/event';
  import { writeText, writeImage } from '@tauri-apps/plugin-clipboard-manager';
  import { save } from '@tauri-apps/plugin-dialog';
  import { appStore } from '$lib/stores.svelte';
  import { AnnotationEngine } from '$lib/AnnotationEngine';
  import type { Annotation, ToolType, Point } from '$lib/types';

  const PALETTE = ['#FF3B30', '#FF9500', '#FFCC00', '#34C759', '#007AFF', '#AF52DE', '#1D1D1F', '#FFFFFF'];

  let canvasContainer = $state<HTMLDivElement | undefined>(undefined);
  let engine = $state<AnnotationEngine | null>(null);
  let canvasWidth = $state(780);
  let canvasHeight = $state(440);

  // Drawing state — plain variables (NOT $state) because they are only
  // read/written inside Konva event handlers, never in the template.
  // Using $state here would trigger Svelte 5 reactivity flushes inside
  // Konva's synchronous event dispatch, which can cause stale reads.
  let isDrawing = false;
  let drawStart: Point | null = null;
  let penPoints: number[] = [];
  let lastMouseEvent: MouseEvent | null = null;

  let unlisten: UnlistenFn | null = null;
  let unlistenNewCapture: UnlistenFn | null = null;
  let currentFilename = $state('');

  const tools: { id: ToolType; label: string; key: string; icon: string }[] = [
    { id: 'rect', label: 'Rectangle', key: 'R', icon: rectIcon() },
    { id: 'ellipse', label: 'Ellipse', key: 'O', icon: ellipseIcon() },
    { id: 'line', label: 'Line', key: 'L', icon: lineIcon() },
    { id: 'arrow', label: 'Arrow', key: 'A', icon: arrowIcon() },
    { id: 'pen', label: 'Pen', key: 'P', icon: penIcon() },
    { id: 'blur', label: 'Blur', key: 'B', icon: blurIcon() },
    { id: 'text', label: 'Text', key: 'T', icon: textIcon() },
    { id: 'step', label: 'Step', key: 'N', icon: stepIcon() },
  ];

  function rectIcon() { return `<svg viewBox="0 0 16 16" fill="none" stroke="currentColor" stroke-width="1.8"><rect x="2" y="3" width="12" height="10" rx="1.5"/></svg>`; }
  function ellipseIcon() { return `<svg viewBox="0 0 16 16" fill="none" stroke="currentColor" stroke-width="1.8"><ellipse cx="8" cy="8" rx="6" ry="5"/></svg>`; }
  function lineIcon() { return `<svg viewBox="0 0 16 16" fill="none" stroke="currentColor" stroke-width="1.8" stroke-linecap="round"><line x1="2" y1="14" x2="14" y2="2"/></svg>`; }
  function arrowIcon() { return `<svg viewBox="0 0 16 16" fill="none" stroke="currentColor" stroke-width="1.8" stroke-linecap="round"><line x1="2" y1="14" x2="12" y2="4"/><polyline points="6,4 12,4 12,10"/></svg>`; }
  function penIcon() { return `<svg viewBox="0 0 16 16" fill="none" stroke="currentColor" stroke-width="1.5"><path d="M11 2l3 3-8 8H3v-3L11 2z"/></svg>`; }
  function blurIcon() { return `<svg viewBox="0 0 16 16" fill="none" stroke="currentColor" stroke-width="1.5"><rect x="2" y="2" width="5" height="5" rx="0.5"/><rect x="9" y="2" width="5" height="5" rx="0.5"/><rect x="2" y="9" width="5" height="5" rx="0.5"/><rect x="9" y="9" width="5" height="5" rx="0.5"/></svg>`; }
  function textIcon() { return `<svg viewBox="0 0 16 16" fill="none" stroke="currentColor" stroke-width="1.5"><line x1="2" y1="3" x2="14" y2="3"/><line x1="8" y1="3" x2="8" y2="14"/><line x1="5" y1="14" x2="11" y2="14"/></svg>`; }
  function stepIcon() { return `<svg viewBox="0 0 16 16" fill="none" stroke="currentColor" stroke-width="1.5"><circle cx="8" cy="8" r="6"/><text x="8" y="12" text-anchor="middle" font-size="8" fill="currentColor" stroke="none" font-weight="bold">1</text></svg>`; }
  function undoIcon() { return `<svg viewBox="0 0 16 16" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round"><path d="M4 8a5 5 0 1 1 .5 2M4 4v4h4"/></svg>`; }
  function redoIcon() { return `<svg viewBox="0 0 16 16" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round"><path d="M12 8a5 5 0 1 0-.5 2M12 4v4H8"/></svg>`; }
  function saveIcon() { return `<svg viewBox="0 0 16 16" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round"><path d="M8 2v8M5 7l3 3 3-3"/><line x1="3" y1="14" x2="13" y2="14"/></svg>`; }
  function copyIcon() { return `<svg viewBox="0 0 16 16" fill="none" stroke="currentColor" stroke-width="1.5"><rect x="5" y="5" width="8" height="8" rx="1"/><path d="M3 11V3h8"/></svg>`; }
  function pinIcon() { return `<svg viewBox="0 0 16 16" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round"><path d="M9 2l5 5-3 1-4 4-1-1 1-2-4-4 1-3zM6 11l-4 4"/></svg>`; }
  function cancelIcon() { return `<svg viewBox="0 0 16 16" fill="none" stroke="currentColor" stroke-width="1.8" stroke-linecap="round"><line x1="4" y1="4" x2="12" y2="12"/><line x1="12" y1="4" x2="4" y2="12"/></svg>`; }
  function doneIcon() { return `<svg viewBox="0 0 16 16" fill="none" stroke="currentColor" stroke-width="1.8" stroke-linecap="round"><polyline points="3,8 7,12 13,4"/></svg>`; }

  // Load an image element and resolve with its natural dimensions.
  function loadImage(src: string): Promise<HTMLImageElement> {
    return new Promise((resolve, reject) => {
      const img = new Image();
      img.onload = () => resolve(img);
      img.onerror = reject;
      img.src = src;
    });
  }

  // Load a new capture: get the image dimensions first, then build the
  // Konva engine synchronously after DOM updates.  Everything after the
  // two awaits is synchronous — no $state writes inside callbacks, which
  // avoids the Svelte 5 reactivity flush that was destroying the Konva
  // stage event handlers before they could be registered.
  async function loadCapture(data: string) {
    appStore.setCaptureImageData(data);
    setFilenameFromNow();

    // 1. Load image to learn dimensions (also yields so Svelte can flush
    //    the {#if captureImageData} DOM block into existence).
    const dataUrl = `data:image/png;base64,${data}`;
    const img = await loadImage(dataUrl);

    // 2. Set wrapper dimensions and wait for DOM to update.
    canvasWidth = Math.min(img.width, 1040);
    canvasHeight = Math.round((canvasWidth / img.width) * img.height);
    await tick();

    // 3. Synchronously create engine, wire events, set background.
    if (!canvasContainer) {
      console.error('canvasContainer unavailable after tick');
      return;
    }
    engine?.destroy();
    engine = new AnnotationEngine(canvasContainer, canvasWidth, canvasHeight);
    setupStageEvents();
    engine.setBaseImage(dataUrl);
  }

  $effect(() => {
    engine?.renderAnnotations(appStore.annotations, appStore.selectedAnnotationId);
  });

  onMount(async () => {
    // Consume any pending capture result stored by the Rust backend.
    try {
      const pending = await invoke<string | null>('consume_capture_result');
      if (pending) await loadCapture(pending);
    } catch (e) {
      console.error('Failed to load initial capture:', e);
    }

    // Keep event listener for future compatibility.
    unlisten = await listen<{ imageData: string; mode: string }>('capture-complete', async (event) => {
      await loadCapture(event.payload.imageData);
    });

    // Fired when the editor window is re-shown for a new capture (the window
    // is hidden rather than destroyed between captures, so onMount does not
    // re-run). Consume the pending result from CaptureStore just like onMount.
    unlistenNewCapture = await listen('new-capture-ready', async () => {
      try {
        const pending = await invoke<string | null>('consume_capture_result');
        if (pending) {
          await loadCapture(pending);
        } else {
          console.warn('new-capture-ready fired but no capture result in store');
        }
      } catch (e) {
        console.error('Failed to consume new capture result:', e);
      }
    });
  });

  onDestroy(() => {
    unlisten?.();
    unlistenNewCapture?.();
    engine?.destroy();
  });

  function setFilenameFromNow() {
    const now = new Date();
    currentFilename = `SnapInk ${now.getFullYear()}-${pad(now.getMonth() + 1)}-${pad(now.getDate())} at ${pad(now.getHours())}.${pad(now.getMinutes())}.${pad(now.getSeconds())}`;
  }

  function pad(n: number) { return String(n).padStart(2, '0'); }

  // Returns the cursor CSS value for the current tool.
  function cursorForTool(tool: ToolType): string {
    switch (tool) {
      case 'select': return 'default';
      case 'text': return 'text';
      case 'pen': return 'crosshair';
      default: return 'crosshair';
    }
  }

  function setupStageEvents() {
    if (!engine) return;
    const stage = engine.stage;
    const container = stage.container();
    container.style.cursor = cursorForTool(appStore.activeTool);

    stage.on('mousedown', (e) => {
      const tool = appStore.activeTool;

      // ── Select mode ───────────────────────────────────────────────
      if (tool === 'select') {
        const isBackground = e.target === stage || e.target?.getLayer() === engine?.baseLayer;
        if (isBackground) {
          appStore.selectAnnotation(null);
        } else if (e.target && e.target !== stage) {
          appStore.selectAnnotation(e.target.id() || null);
        }
        return;
      }

      // ── Text tool — inline editing, no drag ───────────────────────
      if (tool === 'text') {
        const pos = stage.getPointerPosition();
        if (!pos || !engine) return;
        engine.openTextInput(pos.x, pos.y, appStore.activeColor).then((text) => {
          if (text) {
            addAnnotation({
              type: 'text',
              x: pos.x,
              y: pos.y,
              text,
              fontSize: 14,
              bold: false,
              background: true,
            });
          }
          appStore.setActiveTool('select');
          container.style.cursor = cursorForTool('select');
        });
        return;
      }

      // ── Step tool — place on click, no drag ───────────────────────
      if (tool === 'step') {
        const pos = stage.getPointerPosition();
        if (!pos) return;
        addAnnotation({ type: 'step', x: pos.x, y: pos.y, number: appStore.nextStepNumber() });
        appStore.setActiveTool('select');
        container.style.cursor = cursorForTool('select');
        return;
      }

      // ── Drawing tools (rect, ellipse, line, arrow, pen, blur) ─────
      const pos = stage.getPointerPosition();
      if (!pos) return;
      isDrawing = true;
      drawStart = { x: pos.x, y: pos.y };
      if (tool === 'pen') penPoints = [pos.x, pos.y];
    });

    stage.on('mousemove', (e) => {
      lastMouseEvent = e.evt;
      if (!isDrawing || !drawStart || !engine) return;
      const pos = stage.getPointerPosition();
      if (!pos) return;

      const tool = appStore.activeTool;
      const color = appStore.activeColor;
      const sw = appStore.strokeWidth;

      if (tool === 'pen') {
        penPoints = [...penPoints, pos.x, pos.y];
        // Live pen preview
        const previewNode = engine.createNode({
          id: '__preview', type: 'pen', color, strokeWidth: sw, points: penPoints,
        } as Annotation);
        if (previewNode) engine.showPreview(previewNode);
        return;
      }

      // For geometry tools, build a preview annotation.
      const dx = pos.x - drawStart.x;
      const dy = pos.y - drawStart.y;
      if (Math.abs(dx) < 2 && Math.abs(dy) < 2) return;

      let previewAnn: Annotation | null = null;

      if (tool === 'rect') {
        previewAnn = {
          id: '__preview', type: 'rect', color, strokeWidth: sw,
          x: Math.min(drawStart.x, pos.x), y: Math.min(drawStart.y, pos.y),
          width: Math.abs(dx), height: Math.abs(dy), cornerRadius: 0, fill: false,
        } as Annotation;
      } else if (tool === 'ellipse') {
        previewAnn = {
          id: '__preview', type: 'ellipse', color, strokeWidth: sw,
          x: (drawStart.x + pos.x) / 2, y: (drawStart.y + pos.y) / 2,
          radiusX: Math.abs(dx) / 2, radiusY: Math.abs(dy) / 2,
        } as Annotation;
      } else if (tool === 'line') {
        previewAnn = {
          id: '__preview', type: 'line', color, strokeWidth: sw,
          x1: drawStart.x, y1: drawStart.y, x2: pos.x, y2: pos.y,
        } as Annotation;
      } else if (tool === 'arrow') {
        let ex = pos.x, ey = pos.y;
        if (lastMouseEvent?.shiftKey) {
          const angle = Math.round(Math.atan2(dy, dx) / (Math.PI / 4)) * (Math.PI / 4);
          const len = Math.sqrt(dx * dx + dy * dy);
          ex = drawStart.x + Math.cos(angle) * len;
          ey = drawStart.y + Math.sin(angle) * len;
        }
        previewAnn = {
          id: '__preview', type: 'arrow', color, strokeWidth: sw,
          x1: drawStart.x, y1: drawStart.y, x2: ex, y2: ey, filledHead: true,
        } as Annotation;
      } else if (tool === 'blur') {
        previewAnn = {
          id: '__preview', type: 'blur', color, strokeWidth: sw,
          x: Math.min(drawStart.x, pos.x), y: Math.min(drawStart.y, pos.y),
          width: Math.abs(dx), height: Math.abs(dy), strength: 10, mode: 'blur' as const,
        } as Annotation;
      }

      if (previewAnn) {
        const node = engine.createNode(previewAnn);
        if (node) {
          node.listening(false);
          engine.showPreview(node);
        }
      }
    });

    stage.on('mouseup', () => {
      if (!isDrawing || !drawStart || !engine) return;
      const pos = stage.getPointerPosition();
      if (!pos) return;

      engine.clearPreview();
      finalizeDrawing(pos);

      isDrawing = false;
      drawStart = null;
      penPoints = [];

      // Auto-return to select after drawing (Shottr behavior).
      appStore.setActiveTool('select');
      container.style.cursor = cursorForTool('select');
    });
  }

  function finalizeDrawing(end: Point) {
    const start = drawStart!;
    const tool = appStore.activeTool;
    const dx = end.x - start.x;
    const dy = end.y - start.y;

    if (tool === 'rect' && (Math.abs(dx) > 2 || Math.abs(dy) > 2)) {
      addAnnotation({
        type: 'rect',
        x: Math.min(start.x, end.x),
        y: Math.min(start.y, end.y),
        width: Math.abs(dx),
        height: Math.abs(dy),
        cornerRadius: 0,
        fill: false,
      });
    } else if (tool === 'ellipse' && (Math.abs(dx) > 2 || Math.abs(dy) > 2)) {
      addAnnotation({
        type: 'ellipse',
        x: (start.x + end.x) / 2,
        y: (start.y + end.y) / 2,
        radiusX: Math.abs(dx) / 2,
        radiusY: Math.abs(dy) / 2,
      });
    } else if (tool === 'line' && (Math.abs(dx) > 2 || Math.abs(dy) > 2)) {
      addAnnotation({ type: 'line', x1: start.x, y1: start.y, x2: end.x, y2: end.y });
    } else if (tool === 'arrow' && (Math.abs(dx) > 2 || Math.abs(dy) > 2)) {
      let ex = end.x, ey = end.y;
      if (lastMouseEvent?.shiftKey) {
        const angle = Math.round(Math.atan2(dy, dx) / (Math.PI / 4)) * (Math.PI / 4);
        const len = Math.sqrt(dx * dx + dy * dy);
        ex = start.x + Math.cos(angle) * len;
        ey = start.y + Math.sin(angle) * len;
      }
      addAnnotation({ type: 'arrow', x1: start.x, y1: start.y, x2: ex, y2: ey, filledHead: true });
    } else if (tool === 'pen' && penPoints.length > 4) {
      addAnnotation({ type: 'pen', points: penPoints });
    } else if (tool === 'blur' && (Math.abs(dx) > 2 || Math.abs(dy) > 2)) {
      addAnnotation({
        type: 'blur',
        x: Math.min(start.x, end.x),
        y: Math.min(start.y, end.y),
        width: Math.abs(dx),
        height: Math.abs(dy),
        strength: 10,
        mode: 'blur',
      });
    }
  }

  function addAnnotation(partial: Record<string, unknown>) {
    const ann = {
      id: crypto.randomUUID(),
      color: appStore.activeColor,
      strokeWidth: appStore.strokeWidth,
      ...partial,
    } as Annotation;
    appStore.addAnnotation(ann);
  }

  function onKeyDown(e: KeyboardEvent) {
    const tag = (e.target as HTMLElement).tagName;
    if (tag === 'INPUT' || tag === 'TEXTAREA') return;

    if ((e.metaKey || e.ctrlKey) && e.key === 'z' && !e.shiftKey) { e.preventDefault(); appStore.undo(); return; }
    if ((e.metaKey || e.ctrlKey) && e.key === 'z' && e.shiftKey) { e.preventDefault(); appStore.redo(); return; }
    if ((e.metaKey || e.ctrlKey) && e.key === 'c') { e.preventDefault(); copyToClipboard(); return; }
    if ((e.metaKey || e.ctrlKey) && e.key === 's') { e.preventDefault(); saveToFile(); return; }
    if (e.key === 'Escape') { appStore.selectAnnotation(null); return; }
    if (e.key === 'Delete' || e.key === 'Backspace') {
      if (appStore.selectedAnnotationId) appStore.deleteAnnotation(appStore.selectedAnnotationId);
      return;
    }

    const toolMap: Record<string, ToolType> = { r: 'rect', o: 'ellipse', l: 'line', a: 'arrow', p: 'pen', b: 'blur', t: 'text', n: 'step', v: 'select' };
    const mapped = toolMap[e.key.toLowerCase()];
    if (mapped) {
      appStore.setActiveTool(mapped);
      if (engine) engine.stage.container().style.cursor = cursorForTool(mapped);
    }
  }

  async function copyToClipboard() {
    if (!engine) return;
    try {
      const dataUrl = engine.exportToDataURL();
      const base64 = dataUrl.split(',')[1];
      // Convert base64 to Uint8Array for clipboard
      const binary = atob(base64);
      const bytes = new Uint8Array(binary.length);
      for (let i = 0; i < binary.length; i++) bytes[i] = binary.charCodeAt(i);
      await writeImage(bytes);
      appStore.showToast('Copied to clipboard');
    } catch (e) {
      // Fallback: copy as text if image clipboard fails
      try {
        const dataUrl = engine.exportToDataURL();
        await writeText(dataUrl);
        appStore.showToast('Copied to clipboard');
      } catch {
        console.error(e);
      }
    }
  }

  async function saveToFile() {
    if (!engine) return;
    const dataUrl = engine.exportToDataURL();
    const base64 = dataUrl.split(',')[1];
    try {
      const savePath = await save({
        defaultPath: `${currentFilename || 'screenshot'}.png`,
        filters: [{ name: 'PNG Image', extensions: ['png'] }],
      });
      if (savePath) {
        await invoke('export_to_file', { imageData: base64, path: savePath });
        appStore.showToast('Saved');
      }
    } catch (e) {
      console.error(e);
    }
  }

  function openCapture(mode: string) {
    invoke('open_capture_cmd', { mode }).catch(console.error);
  }

  async function pinImage() {
    if (!engine) return;
    const dataUrl = engine.exportToDataURL();
    const base64 = dataUrl.split(',')[1];
    invoke('pin_image', { imageData: base64 }).catch(console.error);
  }

  async function loadFromClipboard() {
    try {
      const data = await invoke<string>('read_clipboard_image');
      await loadCapture(data);
    } catch (e) {
      console.error('No image in clipboard', e);
    }
  }
</script>

<svelte:window onkeydown={onKeyDown} />

<div class="editor-window" data-theme="dark">
  <div class="title-bar">
    <div class="traffic-lights">
      <span class="light close"></span>
      <span class="light minimize"></span>
      <span class="light maximize"></span>
    </div>
    <span class="title-text">
      {currentFilename ? `SnapInk — ${currentFilename}` : 'SnapInk'}
    </span>
  </div>

  <div class="canvas-area">
    {#if appStore.captureImageData}
      <div class="canvas-wrapper" style={`width:${canvasWidth}px;height:${canvasHeight}px`}>
        <div bind:this={canvasContainer} style="width:100%;height:100%"></div>
      </div>
    {:else}
      <div class="empty-state">
        <p>No screenshot yet.</p>
        <p>Use a hotkey or the menu bar icon to capture.</p>
        <button class="load-clipboard-btn" onclick={loadFromClipboard}>Load from Clipboard</button>
      </div>
    {/if}
  </div>

  {#if appStore.captureImageData}
    <!-- Toolbar -->
    <div class="toolbar">
      <!-- Shape tools -->
      <div class="tool-group">
        {#each tools as tool}
          <button
            class="tool-btn"
            class:active={appStore.activeTool === tool.id}
            title={`${tool.label} (${tool.key})`}
            onclick={() => { appStore.setActiveTool(tool.id); if (engine) engine.stage.container().style.cursor = cursorForTool(tool.id); }}
          >
            {@html tool.icon}
          </button>
        {/each}
      </div>

      <div class="toolbar-divider"></div>

      <!-- History -->
      <div class="tool-group">
        <button class="tool-btn" class:disabled={!appStore.canUndo} title="Undo (⌘Z)" onclick={() => appStore.undo()}>
          {@html undoIcon()}
        </button>
        <button class="tool-btn" class:disabled={!appStore.canRedo} title="Redo (⌘⇧Z)" onclick={() => appStore.redo()}>
          {@html redoIcon()}
        </button>
      </div>

      <div class="toolbar-divider"></div>

      <!-- Actions -->
      <div class="tool-group">
        <button class="tool-btn" title="Save (⌘S)" onclick={saveToFile}>
          {@html saveIcon()}
        </button>
        <button class="tool-btn" title="Copy (⌘C)" onclick={copyToClipboard}>
          {@html copyIcon()}
        </button>
        <button class="tool-btn" title="Pin Image" onclick={pinImage}>
          {@html pinIcon()}
        </button>
        <button class="tool-btn danger" title="Cancel (Esc)" onclick={() => appStore.setCaptureImageData(null)}>
          {@html cancelIcon()}
        </button>
        <button class="tool-btn success" title="Done (Enter)" onclick={copyToClipboard}>
          {@html doneIcon()}
        </button>
      </div>
    </div>

    <!-- Color palette bar -->
    <div class="palette-bar">
      {#each PALETTE as color}
        <button
          class="swatch"
          class:active={appStore.activeColor === color}
          style={`background:${color}`}
          title={color}
          onclick={() => appStore.setActiveColor(color)}
        ></button>
      {/each}
      <div class="palette-divider"></div>
      <button class="swatch custom" title="Custom color" onclick={() => {
        const input = document.createElement('input');
        input.type = 'color';
        input.value = appStore.activeColor;
        // Must be in the DOM for WebKit/Tauri to open the native picker.
        input.style.position = 'fixed';
        input.style.opacity = '0';
        input.style.pointerEvents = 'none';
        document.body.appendChild(input);
        input.oninput = () => appStore.setActiveColor(input.value);
        input.onchange = () => { appStore.setActiveColor(input.value); input.remove(); };
        input.addEventListener('blur', () => input.remove(), { once: true });
        input.click();
      }}></button>
    </div>
  {/if}

  <!-- Toast -->
  {#if appStore.toastMessage}
    <div class="toast">
      <span class="toast-check">✓</span>
      {appStore.toastMessage}
    </div>
  {/if}
</div>

<style>
.editor-window {
  width: 100vw;
  height: 100vh;
  background: #1C1C1E;
  display: flex;
  flex-direction: column;
  overflow: hidden;
  position: relative;
}

.title-bar {
  height: 28px;
  background: #2C2C2E;
  border-bottom: 1px solid rgba(255, 255, 255, 0.08);
  display: flex;
  align-items: center;
  padding: 0 12px;
  flex-shrink: 0;
  -webkit-app-region: drag;
}

.traffic-lights {
  display: flex;
  gap: 6px;
  margin-right: 12px;
  -webkit-app-region: no-drag;
}

.light {
  width: 12px;
  height: 12px;
  border-radius: 50%;
}

.light.close { background: #FF5F57; }
.light.minimize { background: #FEBC2E; }
.light.maximize { background: #28C840; }

.title-text {
  flex: 1;
  text-align: center;
  font-size: 12px;
  color: #A1A1A6;
}

.canvas-area {
  flex: 1;
  overflow: auto;
  display: flex;
  align-items: center;
  justify-content: center;
  padding: var(--space-xl);
  padding-bottom: 80px;
  background-image: radial-gradient(circle, rgba(255, 255, 255, 0.06) 1px, transparent 1px);
  background-size: 24px 24px;
}

.canvas-wrapper {
  box-shadow: 0 4px 32px rgba(0, 0, 0, 0.5);
  border-radius: 4px;
  overflow: hidden;
}

.empty-state {
  text-align: center;
  color: #A1A1A6;
  font-size: 14px;
  line-height: 2;
}

.load-clipboard-btn {
  margin-top: 16px;
  padding: 8px 16px;
  background: rgba(10, 132, 255, 0.2);
  color: #0A84FF;
  border: 1px solid rgba(10, 132, 255, 0.3);
  border-radius: var(--radius-md);
  font-size: 13px;
  cursor: pointer;
  font-family: inherit;
}

.toolbar {
  position: absolute;
  bottom: 48px;
  left: 50%;
  transform: translateX(-50%);
  display: flex;
  align-items: center;
  gap: 2px;
  height: 44px;
  background: rgba(44, 44, 46, 0.96);
  border: 1px solid rgba(255, 255, 255, 0.12);
  border-radius: 12px;
  padding: 5px 10px;
  box-shadow: 0 4px 20px rgba(0, 0, 0, 0.5);
}

.tool-group {
  display: flex;
  align-items: center;
  gap: 2px;
}

.toolbar-divider {
  width: 1px;
  height: 18px;
  background: rgba(255, 255, 255, 0.12);
  margin: 0 4px;
}

.tool-btn {
  width: 32px;
  height: 32px;
  display: flex;
  align-items: center;
  justify-content: center;
  border-radius: 6px;
  cursor: pointer;
  color: #F5F5F7;
  background: transparent;
  border: none;
  transition: background 0.1s, color 0.1s;
}

.tool-btn :global(svg) {
  width: 16px;
  height: 16px;
}

.tool-btn:hover {
  background: rgba(255, 255, 255, 0.1);
}

.tool-btn.active {
  background: rgba(10, 132, 255, 0.2);
  color: #0A84FF;
}

.tool-btn.disabled {
  opacity: 0.35;
  cursor: not-allowed;
}

.tool-btn.danger {
  color: #FF453A;
}

.tool-btn.success {
  color: #30D158;
}

.palette-bar {
  position: absolute;
  bottom: 8px;
  left: 50%;
  transform: translateX(-50%);
  display: flex;
  align-items: center;
  gap: var(--space-sm);
  background: rgba(44, 44, 46, 0.92);
  backdrop-filter: blur(20px);
  border: 1px solid rgba(255, 255, 255, 0.1);
  border-radius: 9999px;
  padding: 6px 12px;
}

.swatch {
  width: 18px;
  height: 18px;
  border-radius: 50%;
  border: 1.5px solid rgba(255, 255, 255, 0.15);
  cursor: pointer;
  flex-shrink: 0;
  transition: transform 0.1s;
}

.swatch:hover {
  transform: scale(1.15);
}

.swatch.active {
  box-shadow: 0 0 0 2px #1C1C1E, 0 0 0 4px #0A84FF;
}

.swatch.custom {
  background: conic-gradient(red, orange, yellow, green, blue, purple, red);
  border: none;
}

.palette-divider {
  width: 1px;
  height: 14px;
  background: rgba(255, 255, 255, 0.12);
}

.toast {
  position: absolute;
  bottom: 80px;
  left: 50%;
  transform: translateX(-50%);
  background: rgba(28, 28, 30, 0.92);
  color: white;
  font-size: 13px;
  padding: 8px 16px;
  border-radius: 9999px;
  box-shadow: var(--elevation-pin);
  display: flex;
  align-items: center;
  gap: 6px;
  white-space: nowrap;
  pointer-events: none;
}

.toast-check {
  color: #30D158;
  font-weight: bold;
}
</style>
