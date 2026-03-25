<script lang="ts">
  import { onMount, onDestroy } from 'svelte';
  import { invoke } from '@tauri-apps/api/core';
  import { listen } from '@tauri-apps/api/event';
  import { getCurrentWindow } from '@tauri-apps/api/window';

  let frameCount = $state(0);
  let status = $state<'scrolling' | 'stopping' | 'stitching' | 'error'>('scrolling');
  let errorMsg = $state('');
  let cancelled = false;

  let unlistenFrameAdded: (() => void) | null = null;
  let unlistenDone: (() => void) | null = null;
  let unlistenError: (() => void) | null = null;

  onMount(async () => {
    // Ensure the pill has keyboard focus so Space / Escape are delivered here.
    await getCurrentWindow().setFocus();

    unlistenFrameAdded = await listen<number>('scroll-frame-added', (e) => {
      frameCount = e.payload;
    });

    unlistenDone = await listen<void>('scroll-capture-done', async () => {
      if (!cancelled) await stitch();
    });

    // Surface errors from the Rust capture loop directly in the UI.
    unlistenError = await listen<string>('scroll-capture-error', (e) => {
      if (status === 'scrolling' || status === 'stopping') {
        status = 'error';
        errorMsg = e.payload;
      }
    });

    try {
      await invoke('start_panoramic_capture_cmd');
    } catch (e) {
      status = 'error';
      errorMsg = String(e);
      return;
    }
  });

  onDestroy(() => {
    unlistenFrameAdded?.();
    unlistenDone?.();
    unlistenError?.();
  });

  async function stop() {
    // Give immediate visual feedback before the loop acknowledges the stop.
    status = 'stopping';
    // Signal the Rust loop to stop — it will emit scroll-capture-done
    // which triggers stitch() via the listener above.
    await invoke('stop_scroll_capture_cmd');
  }

  async function stitch() {
    if (frameCount < 1) {
      status = 'error';
      errorMsg = 'No frames captured.';
      return;
    }

    status = 'stitching';

    try {
      const stitched = await invoke<string>('stitch_scroll_frames');
      await invoke('store_capture_result', { data: stitched });
      await invoke('open_editor_cmd');
    } catch (e) {
      status = 'error';
      errorMsg = String(e);
      return;
    }

    // scroll_capture_reset also closes this window.
    await invoke('scroll_capture_reset');
  }

  async function cancel() {
    cancelled = true;
    await invoke('stop_scroll_capture_cmd');
    await invoke('scroll_capture_reset');
  }
</script>

<svelte:window onkeydown={(e) => {
  if (e.key === 'Escape') { cancel(); }
  if (e.key === ' ' && (status === 'scrolling' || status === 'stopping')) { e.preventDefault(); stop(); }
}} />

<div class="pill">
  {#if status === 'scrolling'}
    <span class="icon">↕</span>
    <span class="count">{frameCount} {frameCount === 1 ? 'frame' : 'frames'}</span>
    <span class="hint">· Scroll now · Space to stop</span>
    <div class="divider"></div>
    <button class="btn stop" onclick={stop}>Stop</button>
    <button class="btn cancel" onclick={cancel}>✕</button>
  {:else if status === 'stopping'}
    <span class="icon">⏳</span>
    <span class="label">Stopping…</span>
  {:else if status === 'stitching'}
    <span class="icon">⏳</span>
    <span class="label">Stitching…</span>
  {:else if status === 'error'}
    <span class="icon">⚠</span>
    <span class="label error-text">{errorMsg}</span>
    <div class="divider"></div>
    <button class="btn cancel" onclick={cancel}>✕</button>
  {/if}
</div>

<style>
  :global(html, body) {
    margin: 0;
    padding: 0;
    background: transparent;
    overflow: hidden;
  }

  .pill {
    display: flex;
    align-items: center;
    gap: 6px;
    background: rgba(0, 0, 0, 0.78);
    backdrop-filter: blur(20px);
    -webkit-backdrop-filter: blur(20px);
    border: 1px solid rgba(255, 255, 255, 0.12);
    border-radius: 9999px;
    padding: 0 14px;
    height: 44px;
    box-sizing: border-box;
    margin: 12px;
    box-shadow: 0 4px 24px rgba(0,0,0,0.4);
    cursor: default;
    user-select: none;
    -webkit-app-region: drag;
  }

  .icon {
    font-size: 14px;
    color: rgba(255, 255, 255, 0.7);
  }

  .count {
    font-family: 'SF Mono', 'Menlo', monospace;
    font-size: 13px;
    font-weight: 600;
    color: white;
    min-width: 52px;
  }

  .hint {
    font-size: 12px;
    color: rgba(255, 255, 255, 0.4);
  }

  .label {
    font-size: 13px;
    color: rgba(255, 255, 255, 0.85);
  }

  .error-text {
    color: #FF6B6B;
    font-size: 12px;
    max-width: 160px;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .divider {
    width: 1px;
    height: 16px;
    background: rgba(255, 255, 255, 0.15);
    flex-shrink: 0;
  }

  .btn {
    border: none;
    background: none;
    cursor: pointer;
    font-size: 12px;
    font-weight: 600;
    border-radius: 6px;
    padding: 4px 10px;
    transition: background 0.12s;
    -webkit-app-region: no-drag;
  }

  .btn.stop {
    color: #30D158;
  }

  .btn.stop:hover {
    background: rgba(48, 209, 88, 0.15);
  }

  .btn.cancel {
    color: rgba(255, 255, 255, 0.5);
  }

  .btn.cancel:hover {
    background: rgba(255, 255, 255, 0.08);
    color: white;
  }
</style>
