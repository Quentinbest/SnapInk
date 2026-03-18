<script lang="ts">
  import { onMount, onDestroy } from 'svelte';
  import { invoke } from '@tauri-apps/api/core';
  import { getCurrentWindow } from '@tauri-apps/api/window';

  const appWindow = getCurrentWindow();

  let frameCount = $state(0);
  let status = $state<'capturing' | 'stitching' | 'error'>('capturing');
  let errorMsg = $state('');

  let captureInterval: ReturnType<typeof setInterval> | null = null;

  onMount(() => {
    // Begin capturing frames immediately.
    startCapturing();
  });

  onDestroy(() => {
    stopCapturing();
  });

  function startCapturing() {
    captureInterval = setInterval(async () => {
      try {
        const count = await invoke<number>('scroll_capture_add_frame');
        frameCount = count;
      } catch (e) {
        console.error('Frame capture error:', e);
      }
    }, 400);
  }

  function stopCapturing() {
    if (captureInterval !== null) {
      clearInterval(captureInterval);
      captureInterval = null;
    }
  }

  async function done() {
    stopCapturing();

    if (frameCount < 1) {
      status = 'error';
      errorMsg = 'Scroll the content first, then click Done.';
      // Resume capturing so user can try again.
      startCapturing();
      return;
    }

    status = 'stitching';

    try {
      // Single frame: skip stitching, use it directly.
      const stitched = await invoke<string>('stitch_scroll_frames');
      await invoke('store_capture_result', { data: stitched });
      await invoke('open_editor_cmd');
    } catch (e) {
      status = 'error';
      errorMsg = String(e);
      return;
    }

    // Clean up state and close this window (reset also closes us).
    await invoke('scroll_capture_reset');
  }

  async function cancel() {
    stopCapturing();
    await invoke('scroll_capture_reset');
    // scroll_capture_reset closes the scroll-control window via Rust.
  }
</script>

<svelte:window onkeydown={(e) => e.key === 'Escape' && cancel()} />

<div class="pill">
  {#if status === 'capturing'}
    <span class="icon">↕</span>
    <span class="count">{frameCount} {frameCount === 1 ? 'frame' : 'frames'}</span>
    <div class="divider"></div>
    <button class="btn done" onclick={done}>Done</button>
    <button class="btn cancel" onclick={cancel}>✕</button>
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
    /* Enable drag to reposition the window */
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

  .btn.done {
    color: #30D158;
  }

  .btn.done:hover {
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
