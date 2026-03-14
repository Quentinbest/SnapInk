<script lang="ts">
  import '../app.css';
  import { onMount } from 'svelte';
  import { invoke } from '@tauri-apps/api/core';
  import { appStore } from '$lib/stores.svelte';
  import type { Settings } from '$lib/types';

  let { children } = $props();

  onMount(async () => {
    try {
      const settings = await invoke<Settings>('get_settings');
      appStore.setSettings(settings);
      applyTheme(settings.ui.theme);
    } catch {
      applyTheme('system');
    }
  });

  function applyTheme(theme: string) {
    const prefersDark = window.matchMedia('(prefers-color-scheme: dark)').matches;
    const isDark = theme === 'dark' || (theme === 'system' && prefersDark);
    document.documentElement.setAttribute('data-theme', isDark ? 'dark' : 'light');
  }
</script>

{@render children()}
