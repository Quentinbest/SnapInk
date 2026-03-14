<script lang="ts">
  import { appStore } from './stores.svelte';
  import type { Annotation } from './types';

  let { annotation, x = 0, y = 0 }: {
    annotation: Annotation | null;
    x?: number;
    y?: number;
  } = $props();

  function changeColor() {
    const input = document.createElement('input');
    input.type = 'color';
    input.value = annotation?.color ?? '#FF3B30';
    input.onchange = () => {
      if (annotation) appStore.updateAnnotation(annotation.id, { color: input.value });
    };
    input.click();
  }

  function duplicate() {
    if (!annotation) return;
    const newAnn = { ...annotation, id: crypto.randomUUID() } as Annotation;
    appStore.addAnnotation(newAnn);
  }

  function deleteAnn() {
    if (annotation) appStore.deleteAnnotation(annotation.id);
  }
</script>

{#if annotation}
  <div class="contextual-bar" style={`left:${x}px;top:${y}px`}>
    <button class="ctx-btn" title="Change color" onclick={changeColor}>
      <span class="color-dot" style={`background:${annotation.color}`}></span>
    </button>
    <div class="ctx-divider"></div>
    <button class="ctx-btn" title="Duplicate" onclick={duplicate}>⎘</button>
    <div class="ctx-divider"></div>
    <button class="ctx-btn danger" title="Delete" onclick={deleteAnn}>🗑</button>
  </div>
{/if}

<style>
.contextual-bar {
  position: absolute;
  display: flex;
  align-items: center;
  gap: 2px;
  background: rgba(44, 44, 46, 0.98);
  border: 1px solid rgba(255, 255, 255, 0.12);
  border-radius: var(--radius-md);
  padding: 4px 6px;
  box-shadow: var(--elevation-popover);
  transform: translateY(-100%) translateY(-8px);
  z-index: 100;
  pointer-events: all;
}

.ctx-btn {
  width: 28px;
  height: 28px;
  display: flex;
  align-items: center;
  justify-content: center;
  border-radius: 5px;
  cursor: pointer;
  color: #F5F5F7;
  background: transparent;
  border: none;
  font-size: 13px;
}

.ctx-btn:hover {
  background: rgba(255, 255, 255, 0.1);
}

.ctx-btn.danger:hover {
  background: rgba(255, 69, 58, 0.15);
  color: #FF453A;
}

.color-dot {
  width: 14px;
  height: 14px;
  border-radius: 50%;
  border: 1.5px solid rgba(255, 255, 255, 0.3);
  display: block;
}

.ctx-divider {
  width: 1px;
  height: 14px;
  background: rgba(255, 255, 255, 0.12);
  margin: 0 2px;
}
</style>
