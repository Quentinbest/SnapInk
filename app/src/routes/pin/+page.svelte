<script lang="ts">
  import { onMount, onDestroy } from 'svelte';
  import { invoke } from '@tauri-apps/api/core';
  import { getCurrentWindow } from '@tauri-apps/api/window';

  let imageData = $state<string | null>(null);
  let opacity = $state(85);
  let showControls = $state(false);
  let winId = $state('');
  const appWindow = getCurrentWindow();

  onMount(async () => {
    const params = new URLSearchParams(window.location.search);
    const id = params.get('id') ?? '';
    winId = id;
    if (id) {
      imageData = await invoke<string | null>('get_pin_image', { id });
    }
  });

  onDestroy(async () => {
    if (winId) {
      await invoke('remove_pin_image', { id: winId }).catch(() => {});
    }
  });

  async function close() {
    await appWindow.close();
  }
</script>

<div
  class="pin-window"
  style={`opacity:${opacity / 100}`}
  role="presentation"
  onmouseenter={() => showControls = true}
  onmouseleave={() => showControls = false}
>
  {#if imageData}
    <img src={`data:image/png;base64,${imageData}`} alt="Pinned screenshot" class="pin-image" draggable={false} />
  {:else}
    <div class="empty">Loading…</div>
  {/if}

  {#if showControls}
    <button class="close-btn" onclick={close} aria-label="Close">✕</button>
    <div class="opacity-label">{opacity}%</div>
    <input
      class="opacity-slider"
      type="range"
      min="20"
      max="100"
      bind:value={opacity}
      aria-label="Opacity"
    />
  {/if}
</div>

<style>
.pin-window {
  width: 100vw;
  height: 100vh;
  position: relative;
  overflow: hidden;
  background: transparent;
  cursor: default;
}

.pin-image {
  width: 100%;
  height: 100%;
  object-fit: contain;
  display: block;
  pointer-events: none;
}

.empty {
  display: flex;
  align-items: center;
  justify-content: center;
  height: 100%;
  color: rgba(255, 255, 255, 0.5);
  font-size: 12px;
}

.close-btn {
  position: absolute;
  top: 8px;
  left: 8px;
  width: 20px;
  height: 20px;
  background: #FF3B30;
  color: white;
  border: none;
  border-radius: 50%;
  font-size: 10px;
  font-weight: bold;
  cursor: pointer;
  display: flex;
  align-items: center;
  justify-content: center;
}

.opacity-label {
  position: absolute;
  bottom: 28px;
  left: 50%;
  transform: translateX(-50%);
  background: rgba(0, 0, 0, 0.6);
  color: white;
  font-size: 10px;
  padding: 2px 8px;
  border-radius: 9999px;
  pointer-events: none;
  white-space: nowrap;
}

.opacity-slider {
  position: absolute;
  bottom: 8px;
  left: 10%;
  width: 80%;
}
</style>
