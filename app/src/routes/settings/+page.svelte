<script lang="ts">
  import { onMount } from 'svelte';
  import { invoke } from '@tauri-apps/api/core';
  import { appStore } from '$lib/stores.svelte';
  import type { Settings } from '$lib/types';

  let activeTab = $state<'general' | 'shortcuts' | 'output'>('general');
  let settings = $state<Settings>(appStore.settings);
  let saveStatus = $state<string | null>(null);

  onMount(async () => {
    try {
      const loaded = await invoke<Settings>('get_settings');
      settings = { ...loaded };
      appStore.setSettings(loaded);
    } catch (e) {
      console.error(e);
    }
  });

  async function save() {
    try {
      await invoke('save_settings', { settings });
      appStore.setSettings(settings);
      saveStatus = 'Saved';
      setTimeout(() => { saveStatus = null; }, 2000);
    } catch (e) {
      saveStatus = 'Error saving';
    }
  }

  const ACTION_LABELS: Record<string, string> = {
    capture_area: 'Capture Area',
    capture_screen: 'Capture Screen',
    capture_window: 'Capture Window',
    capture_scrolling: 'Scrolling Capture',
    capture_ocr: 'Recognize Text (OCR)',
    repeat_last: 'Repeat Last Capture',
  };
</script>

<div class="settings-window">
  <div class="title-bar">
    <span class="title">Settings</span>
  </div>

  <div class="tab-bar">
    <button class="tab" class:active={activeTab === 'general'} onclick={() => activeTab = 'general'}>General</button>
    <button class="tab" class:active={activeTab === 'shortcuts'} onclick={() => activeTab = 'shortcuts'}>Shortcuts</button>
    <button class="tab" class:active={activeTab === 'output'} onclick={() => activeTab = 'output'}>Output</button>
  </div>

  <div class="tab-content">
    {#if activeTab === 'general'}
      <section class="settings-section">
        <h2 class="section-title">Startup</h2>
        <label class="setting-row">
          <span>Launch at Login</span>
          <input type="checkbox" bind:checked={settings.ui.launchAtLogin} />
        </label>
        <label class="setting-row">
          <span>Show menu bar icon</span>
          <input type="checkbox" bind:checked={settings.ui.showMenuBarIcon} />
        </label>
        <label class="setting-row">
          <span>Play sound on capture</span>
          <input type="checkbox" bind:checked={settings.capture.playSoundOnCapture} />
        </label>
      </section>

      <section class="settings-section">
        <h2 class="section-title">After Capture</h2>
        <label class="setting-row">
          <input type="radio" bind:group={settings.afterCapture} value="open_editor" />
          <span>Open in annotation editor</span>
        </label>
        <label class="setting-row">
          <input type="radio" bind:group={settings.afterCapture} value="copy_clipboard" />
          <span>Copy to clipboard immediately</span>
        </label>
        <label class="setting-row">
          <span>Also copy after annotating</span>
          <input type="checkbox" bind:checked={settings.alsoCopyAfterAnnotating} />
        </label>
      </section>

      <section class="settings-section">
        <h2 class="section-title">Appearance</h2>
        <label class="setting-row">
          <span>Theme</span>
          <select bind:value={settings.ui.theme}>
            <option value="system">System</option>
            <option value="light">Light</option>
            <option value="dark">Dark</option>
          </select>
        </label>
      </section>
    {/if}

    {#if activeTab === 'shortcuts'}
      <section class="settings-section">
        <h2 class="section-title">Capture Shortcuts</h2>
        {#each settings.hotkeys as hotkey}
          <div class="setting-row">
            <span>{ACTION_LABELS[hotkey.action] ?? hotkey.action}</span>
            <div class="kbd-row">
              {#each hotkey.shortcut.split('+') as part}
                <kbd>{part.replace('CommandOrControl', '⌘').replace('Ctrl', '⌃').replace('Shift', '⇧').replace('Control', '⌃').replace('Alt', '⌥')}</kbd>
              {/each}
            </div>
          </div>
        {/each}
        <p class="note">ⓘ Hotkey recording coming soon.</p>
      </section>
    {/if}

    {#if activeTab === 'output'}
      <section class="settings-section">
        <h2 class="section-title">Save Location</h2>
        <div class="setting-row">
          <span class="path-display">{settings.output.savePath || '~/Desktop'}</span>
          <button class="btn-secondary">Change…</button>
        </div>
      </section>

      <section class="settings-section">
        <h2 class="section-title">Filename</h2>
        <label class="setting-col">
          <span>Pattern</span>
          <input class="text-input" type="text" bind:value={settings.output.filenamePattern} />
        </label>
      </section>

      <section class="settings-section">
        <h2 class="section-title">Format</h2>
        <label class="setting-row">
          <span>File Format</span>
          <select bind:value={settings.output.format}>
            <option value="png">PNG</option>
            <option value="jpg">JPEG</option>
            <option value="webp">WebP</option>
          </select>
        </label>
        {#if settings.output.format === 'jpg'}
          <label class="setting-row">
            <span>JPEG Quality ({settings.output.jpegQuality}%)</span>
            <input type="range" min="1" max="100" bind:value={settings.output.jpegQuality} />
          </label>
        {/if}
        <label class="setting-row">
          <span>Copy at 2× (Retina) resolution</span>
          <input type="checkbox" bind:checked={settings.output.retinaClipboard} />
        </label>
      </section>
    {/if}
  </div>

  <div class="footer">
    {#if saveStatus}
      <span class="save-status">{saveStatus}</span>
    {/if}
    <button class="btn-primary" onclick={save}>Save</button>
  </div>
</div>

<style>
.settings-window {
  width: 100vw;
  height: 100vh;
  background: var(--bg-primary);
  display: flex;
  flex-direction: column;
  overflow: hidden;
}

.title-bar {
  height: 40px;
  background: #F2F2F7;
  border-bottom: 1px solid rgba(0, 0, 0, 0.08);
  display: flex;
  align-items: center;
  justify-content: center;
  flex-shrink: 0;
  -webkit-app-region: drag;
}

.title {
  font-size: 16px;
  font-weight: 600;
  letter-spacing: -0.2px;
  color: var(--text-primary);
}

.tab-bar {
  display: flex;
  border-bottom: 1px solid var(--border-subtle);
  padding: 8px 16px 0;
  gap: 2px;
}

.tab {
  padding: 6px 14px;
  border-radius: 7px 7px 0 0;
  font-size: 13px;
  color: #86868B;
  background: white;
  border: 1px solid #D1D1D6;
  border-bottom: none;
  cursor: pointer;
}

.tab.active {
  background: #007AFF;
  color: white;
  border-color: #007AFF;
}

.tab-content {
  flex: 1;
  overflow-y: auto;
  padding: var(--space-lg);
}

.settings-section {
  margin-bottom: var(--space-xl);
}

.section-title {
  font-size: 11px;
  font-weight: 600;
  text-transform: uppercase;
  letter-spacing: 0.04em;
  color: var(--text-secondary);
  margin-bottom: var(--space-sm);
}

.setting-row {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 8px 0;
  border-bottom: 1px solid var(--border-subtle);
  gap: var(--space-md);
  font-size: 13px;
  color: var(--text-primary);
  cursor: default;
}

.setting-col {
  display: flex;
  flex-direction: column;
  gap: var(--space-xs);
  padding: 8px 0;
  font-size: 13px;
  color: var(--text-primary);
}

.kbd-row {
  display: flex;
  gap: 3px;
}

kbd {
  background: #F2F2F7;
  border: 1px solid #D1D1D6;
  border-radius: 4px;
  font-size: 11px;
  padding: 2px 5px;
  font-family: 'SF Mono', 'Menlo', monospace;
}

.path-display {
  font-size: 12px;
  color: var(--text-secondary);
  font-family: 'SF Mono', 'Menlo', monospace;
}

.text-input {
  width: 100%;
  padding: 6px 8px;
  border: 1px solid #D1D1D6;
  border-radius: 6px;
  font-size: 13px;
  font-family: inherit;
  color: var(--text-primary);
  background: var(--bg-primary);
}

select {
  padding: 4px 8px;
  border: 1px solid #D1D1D6;
  border-radius: 6px;
  font-size: 13px;
  font-family: inherit;
  background: var(--bg-primary);
  color: var(--text-primary);
}

.note {
  font-size: 11px;
  color: var(--text-secondary);
  margin-top: var(--space-sm);
}

.footer {
  height: 48px;
  border-top: 1px solid var(--border-subtle);
  display: flex;
  align-items: center;
  justify-content: flex-end;
  padding: 0 var(--space-lg);
  gap: var(--space-md);
  flex-shrink: 0;
}

.save-status {
  font-size: 12px;
  color: var(--success);
}

.btn-primary {
  padding: 6px 16px;
  background: #007AFF;
  color: white;
  border: none;
  border-radius: 6px;
  font-size: 13px;
  font-weight: 500;
  cursor: pointer;
  font-family: inherit;
}

.btn-primary:hover {
  background: #0066CC;
}

.btn-secondary {
  padding: 4px 10px;
  background: white;
  color: #007AFF;
  border: 1px solid #D1D1D6;
  border-radius: 6px;
  font-size: 12px;
  cursor: pointer;
  font-family: inherit;
}
</style>
