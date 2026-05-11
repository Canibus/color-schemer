<!-- src/lib/ProfileEditor.svelte -->
<script lang="ts">
  import type { DisplayInfo, DisplayProfile, DisplaySettings, ProcessInfo } from "./types";
  import { i18n } from "./i18n.svelte";
  import { invoke } from "@tauri-apps/api/tauri";
  import { untrack } from "svelte";
  
  let { 
    profile, 
    displays = [],
    onSave, 
    onCancel, 
    onPreview, 
    onDelete = undefined 
  } = $props<{
    profile: DisplayProfile;
    displays?: DisplayInfo[];
    onSave: (updated: DisplayProfile) => void;
    onCancel: () => void;
    onPreview: (settings: DisplaySettings, displayIds: string[]) => void;
    onDelete?: () => void;
  }>();

  // Local copy for editing
  function createEditableCopy(p: DisplayProfile): DisplayProfile {
      const copy = $state.snapshot(p);
      if (!copy.target_displays) copy.target_displays = [];
      if (!copy.applications) copy.applications = [];
      return copy;
  }

  let edited = $state(untrack(() => createEditableCopy(profile)));
  
  // App picker state
  let runningApps = $state<ProcessInfo[]>([]);
  let showAppPicker = $state(false);
  let appPickerLoading = $state(false);

  $effect(() => {
    // We update 'edited' only when the 'profile' prop changes (e.g. switching which profile is being edited)
    edited = createEditableCopy(profile);
  });

  function handleInput() {
      onPreview(edited.settings, edited.target_displays);
  }

  function toggleDisplay(id: string) {
      if (edited.target_displays.includes(id)) {
          edited.target_displays = edited.target_displays.filter(d => d !== id);
      } else {
          edited.target_displays = [...edited.target_displays, id];
      }
  }

  async function openAppPicker() {
      showAppPicker = true;
      appPickerLoading = true;
      try {
          runningApps = await invoke<ProcessInfo[]>("get_running_apps");
      } catch (e) {
          console.error("Failed to fetch running apps:", e);
      } finally {
          appPickerLoading = false;
      }
  }

  function addApplication(name: string) {
      if (!edited.applications.includes(name)) {
          edited.applications = [...edited.applications, name];
      }
      showAppPicker = false;
  }

  function removeApplication(name: string) {
      edited.applications = edited.applications.filter(a => a !== name);
  }
</script>

<div class="editor">
  <h3>{i18n.t('editor.title')}</h3>
  
  <div class="field-group">
    <div class="group-label">{i18n.t('editor.info')}</div>
    <div class="group-content">
      <label>{i18n.t('editor.name')}
          <input type="text" bind:value={edited.name} maxlength="32" />
      </label>
      <label>{i18n.t('editor.description')}
          <input type="text" bind:value={edited.description} maxlength="32" />
      </label>
    </div>
  </div>

  <div class="field-group">
    <div class="group-label">{i18n.t('editor.displays')}</div>
    <div class="group-content">
      <div class="display-list">
        {#each displays as d}
          <label class="display-item">
            <input type="checkbox" checked={edited.target_displays.includes(d.id)} onchange={() => toggleDisplay(d.id)} />
            <div class="display-info">
              <span class="display-name">{d.name} {d.is_primary ? `(${i18n.t('profiles.primary')})` : ''}</span>
              {#if d.id !== d.name}
                <span class="display-id">{d.id}</span>
              {/if}
            </div>
          </label>
        {/each}
        {#if displays.length === 0}
          <div class="no-displays">{i18n.t('editor.all_displays')}</div>
        {/if}
      </div>
    </div>
  </div>

  <div class="field-group">
    <div class="group-label">{i18n.t('editor.applications')}</div>
    <div class="group-content">
      <div class="apps-help">{i18n.t('editor.apps_help')}</div>
      <div class="app-list">
        {#each edited.applications as app}
          <div class="app-tag">
            <span>{app}</span>
            <button class="remove-app" onclick={() => removeApplication(app)}>×</button>
          </div>
        {/each}
        {#if edited.applications.length === 0}
          <div class="no-apps">{i18n.t('editor.no_apps')}</div>
        {/if}
      </div>
      <button class="add-app-btn" onclick={openAppPicker}>
        {i18n.t('editor.pick_app')}
      </button>
    </div>
  </div>

  <div class="field-group">
      <div class="group-label">{i18n.t('editor.settings')}</div>
      <div class="group-content">
          <label>
              <div class="label-header">
                <span>{i18n.t('editor.brightness')}</span>
                <span class="value-display">{edited.settings.brightness}</span>
              </div>
              <input type="range" min="0" max="2" step="0.01" bind:value={edited.settings.brightness} oninput={handleInput} />
          </label>
          <label>
              <div class="label-header">
                <span>{i18n.t('editor.contrast')}</span>
                <span class="value-display">{edited.settings.contrast}</span>
              </div>
              <input type="range" min="0" max="2" step="0.01" bind:value={edited.settings.contrast} oninput={handleInput} />
          </label>
          <label>
              <div class="label-header">
                <span>{i18n.t('editor.gamma')}</span>
                <span class="value-display">{edited.settings.gamma}</span>
              </div>
              <input type="range" min="0.1" max="3" step="0.01" bind:value={edited.settings.gamma} oninput={handleInput} />
          </label>
          <label>
              <div class="label-header">
                <span>{i18n.t('editor.vibrance')}</span>
                <span class="value-display">{edited.settings.digital_vibrance}</span>
              </div>
              <input type="range" min="-50" max="100" step="1" bind:value={edited.settings.digital_vibrance} oninput={handleInput} />
          </label>
      </div>
  </div>

  <div class="actions">
      {#if onDelete}
        <button class="delete-btn" onclick={onDelete}>{i18n.t('editor.delete')}</button>
      {/if}
      <button onclick={onCancel}>{i18n.t('editor.cancel')}</button>
      <button onclick={() => onSave(edited)}>{i18n.t('editor.save')}</button>
  </div>
</div>

{#if showAppPicker}
  <div 
    class="modal-overlay" 
    onclick={() => showAppPicker = false} 
    onkeydown={(e) => { if (e.key === 'Enter' || e.key === ' ') showAppPicker = false; }}
    role="button" 
    tabindex="0"
  >
    <div 
      class="modal-content" 
      onclick={(e) => e.stopPropagation()} 
      onkeydown={(e) => e.stopPropagation()}
      role="button"
      tabindex="-1"
    >
      <div class="modal-header">
        <h4>{i18n.t('editor.pick_app')}</h4>
        <div class="header-actions">
          <button class="refresh-btn" onclick={openAppPicker} disabled={appPickerLoading} title="Refresh">
            <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M23 4v6h-6M1 20v-6h6M3.51 9a9 9 0 0 1 14.85-3.36L23 10M1 14l4.64 4.36A9 9 0 0 0 20.49 15"/></svg>
          </button>
          <button class="close-modal" onclick={() => showAppPicker = false}>×</button>
        </div>
      </div>
      <div class="modal-body">
        {#if appPickerLoading}
          <div class="loading">{i18n.t('app.status.loading')}</div>
        {:else if runningApps.length === 0}
          <div class="no-apps-found">
            {i18n.t('editor.no_apps_found')}
          </div>
        {:else}
          <div class="running-apps">
            {#each runningApps as app}
              <button class="app-item" onclick={() => addApplication(app.name)}>
                <span class="app-name">{app.name}</span>
                <span class="app-title">{app.title}</span>
              </button>
            {/each}
          </div>
        {/if}
      </div>
    </div>
  </div>
{/if}

<style>
  .editor { display: flex; flex-direction: column; gap: 15px; color: var(--text-main); }
  h3 { color: var(--primary); text-transform: uppercase; font-size: 16px; margin-top: 0; letter-spacing: 1px; font-family: var(--font-mono); }
  
  label { display: flex; flex-direction: column; gap: 8px; font-family: var(--font-mono); font-size: 12px; text-transform: uppercase; color: var(--text-muted); }
  
  input[type="range"] {
    accent-color: var(--primary);
    cursor: pointer;
    height: 6px;
    border-radius: 3px;
    background: var(--bg-surface);
  }
  
  .value-display {
    font-family: var(--font-mono);
    color: var(--primary);
    font-weight: bold;
    font-size: 13px;
  }

  .label-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    width: 100%;
  }
  
  .field-group {
    background: var(--bg-layer);
    border: 1px solid var(--border);
    border-radius: 10px;
    margin-bottom: 10px;
    overflow: hidden;
  }
  
  .group-label {
    background: var(--bg-active);
    color: var(--primary);
    font-family: var(--font-mono);
    font-size: 11px;
    text-transform: uppercase;
    padding: 8px 14px;
    border-bottom: 1px solid var(--border);
    letter-spacing: 1px;
  }

  .group-content {
    padding: 20px;
    display: flex;
    flex-direction: column;
    gap: 18px;
  }
  
  input[type="text"] {
    background: var(--bg-surface);
    border: 1px solid var(--border);
    color: var(--text-main);
    padding: 10px 12px;
    border-radius: 8px;
    font-family: var(--font-sans);
    transition: all 0.2s;
  }
  
  input[type="text"]:focus {
    border-color: var(--primary);
    outline: none;
    box-shadow: 0 0 5px var(--primary-glow), inset 0 0 5px rgba(0,0,0,0.5);
  }
  
  .actions { display: flex; justify-content: flex-end; gap: 12px; margin-top: 10px; }

  .display-list {
    display: flex;
    flex-direction: column;
    gap: 10px;
  }

  .display-item {
    display: flex;
    flex-direction: row;
    align-items: center;
    gap: 12px;
    padding: 10px;
    background: var(--bg-surface);
    border: 1px solid var(--border);
    border-radius: 8px;
    cursor: pointer;
    transition: all 0.2s;
  }

  .display-item:hover {
    border-color: var(--primary);
    background: var(--bg-active);
  }

  .display-item input[type="checkbox"] {
    appearance: none;
    width: 18px;
    height: 18px;
    border: 2px solid var(--border);
    border-radius: 4px;
    cursor: pointer;
    position: relative;
    transition: all 0.2s;
    background: var(--bg-layer);
    flex-shrink: 0;
  }

  .display-item input[type="checkbox"]:checked {
    background: var(--primary);
    border-color: var(--primary);
    box-shadow: 0 0 5px var(--primary-glow);
  }

  .display-item input[type="checkbox"]:checked::after {
    content: '✓';
    position: absolute;
    top: 50%;
    left: 50%;
    transform: translate(-50%, -50%);
    color: var(--bg-surface);
    font-size: 12px;
    font-weight: bold;
  }

  .display-info {
    display: flex;
    flex-direction: column;
    gap: 2px;
  }

  .display-name {
    color: var(--text-main);
    font-size: 13px;
    font-weight: 500;
  }

  .display-id {
    color: var(--text-muted);
    font-size: 11px;
    font-family: var(--font-mono);
  }

  .no-displays {
    color: var(--text-muted);
    font-size: 12px;
    font-style: italic;
    text-align: center;
    padding: 10px;
  }
  
  .delete-btn { 
    background: var(--bg-surface); 
    border: 1px solid var(--error-dim); 
    color: var(--error); 
    border-radius: 8px; 
    padding: 8px 18px;
    margin-right: auto;
    cursor: pointer;
    font-family: var(--font-mono);
    text-transform: uppercase;
    font-size: 12px;
    letter-spacing: 1px;
    transition: all 0.2s;
    box-shadow: inset 0 -2px 0 var(--error-dim);
  }
  
  .delete-btn:hover { 
    background: var(--error-dim); 
    color: var(--text-main);
    box-shadow: 0 0 10px var(--error-glow), inset 0 -1px 0 var(--error-dim); 
    transform: translateY(-1px);
  }

  .delete-btn:active {
    transform: translateY(1px);
    box-shadow: none;
  }

  button:not(.delete-btn) {
    background: var(--bg-surface);
    border: 1px solid var(--border);
    color: var(--text-main);
    padding: 8px 20px;
    border-radius: 8px;
    cursor: pointer;
    font-family: var(--font-mono);
    text-transform: uppercase;
    font-size: 12px;
    letter-spacing: 1px;
    transition: all 0.2s;
  }

  button:not(.delete-btn):hover {
    border-color: var(--primary);
    color: var(--primary);
    background: var(--bg-active);
    transform: translateY(-1px);
  }
  
  button:not(.delete-btn):last-child {
    background: var(--primary);
    color: var(--bg-surface);
    border-color: var(--primary);
    font-weight: bold;
    box-shadow: inset 0 -2px 0 var(--primary-dim);
  }
  
  button:not(.delete-btn):last-child:hover {
    filter: brightness(1.1);
    box-shadow: var(--glow-shadow), inset 0 -1px 0 var(--primary-dim);
  }

  button:not(.delete-btn):active {
    transform: translateY(1px);
    box-shadow: none;
  }

  /* Application Picker & List Styles */
  .apps-help {
    font-size: 11px;
    color: var(--text-muted);
    font-style: italic;
    margin-bottom: -5px;
  }

  .app-list {
    display: flex;
    flex-wrap: wrap;
    gap: 8px;
    min-height: 40px;
    padding: 10px;
    background: var(--bg-surface);
    border: 1px solid var(--border);
    border-radius: 8px;
  }

  .app-tag {
    display: flex;
    align-items: center;
    gap: 8px;
    padding: 4px 10px;
    background: var(--bg-active);
    border: 1px solid var(--primary-dim);
    border-radius: 6px;
    font-family: var(--font-mono);
    font-size: 12px;
    color: var(--primary);
  }

  .remove-app {
    background: none !important;
    border: none !important;
    color: var(--text-muted) !important;
    padding: 0 !important;
    font-size: 16px !important;
    line-height: 1 !important;
    cursor: pointer !important;
    box-shadow: none !important;
    transform: none !important;
    margin: 0 !important;
    min-width: unset !important;
  }

  .remove-app:hover {
    color: var(--error) !important;
  }

  .no-apps {
    color: var(--text-muted);
    font-size: 12px;
    font-style: italic;
    margin: auto;
  }

  .add-app-btn {
    align-self: flex-start;
    box-shadow: none !important;
    border-bottom: 2px solid var(--primary-dim) !important;
  }

  /* Modal Styles */
  .modal-overlay {
    position: fixed;
    top: 0;
    left: 0;
    width: 100%;
    height: 100%;
    background: rgba(0, 0, 0, 0.85);
    backdrop-filter: blur(4px);
    display: flex;
    align-items: center;
    justify-content: center;
    z-index: 1000;
  }

  .modal-content {
    background: var(--bg-layer);
    border: 1px solid var(--primary);
    border-radius: 12px;
    width: 90%;
    max-width: 500px;
    max-height: 80vh;
    display: flex;
    flex-direction: column;
    box-shadow: 0 0 30px rgba(0, 0, 0, 0.5), 0 0 15px var(--primary-glow);
    animation: modal-appear 0.3s ease-out;
  }

  @keyframes modal-appear {
    from { opacity: 0; transform: scale(0.95) translateY(10px); }
    to { opacity: 1; transform: scale(1) translateY(0); }
  }

  .modal-header {
    padding: 15px 20px;
    border-bottom: 1px solid var(--border);
    display: flex;
    justify-content: space-between;
    align-items: center;
    background: var(--bg-active);
  }

  .modal-header h4 {
    margin: 0;
    color: var(--primary);
    font-family: var(--font-mono);
    text-transform: uppercase;
    letter-spacing: 1px;
    font-size: 14px;
  }

  .header-actions {
    display: flex;
    align-items: center;
    gap: 12px;
  }

  .refresh-btn {
    background: none !important;
    border: none !important;
    color: var(--text-muted) !important;
    padding: 0 !important;
    cursor: pointer !important;
    box-shadow: none !important;
    transform: none !important;
    margin: 0 !important;
    min-width: unset !important;
    display: flex;
    align-items: center;
    justify-content: center;
    transition: color 0.2s;
  }

  .refresh-btn:hover:not(:disabled) {
    color: var(--primary) !important;
  }

  .refresh-btn:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }

  .refresh-btn:disabled svg {
    animation: spin 1s linear infinite;
  }

  @keyframes spin {
    from { transform: rotate(0deg); }
    to { transform: rotate(360deg); }
  }

  .close-modal {
    background: none !important;
    border: none !important;
    color: var(--text-muted) !important;
    font-size: 24px !important;
    padding: 0 !important;
    cursor: pointer !important;
    box-shadow: none !important;
    transform: none !important;
    margin: 0 !important;
    min-width: unset !important;
  }

  .close-modal:hover {
    color: var(--primary) !important;
  }

  .modal-body {
    padding: 0;
    overflow-y: auto;
    flex: 1;
    border-bottom-left-radius: 12px;
    border-bottom-right-radius: 12px;
  }

  /* Custom Industrial Scrollbar */
  .modal-body::-webkit-scrollbar {
    width: 8px;
  }

  .modal-body::-webkit-scrollbar-track {
    background: var(--bg-layer);
  }

  .modal-body::-webkit-scrollbar-thumb {
    background: var(--border);
    border-radius: 4px;
    border: 2px solid var(--bg-layer);
  }

  .modal-body::-webkit-scrollbar-thumb:hover {
    background: var(--primary-dim);
  }

  .running-apps {
    display: flex;
    flex-direction: column;
    border: 1px solid var(--border);
    margin: 10px;
    border-radius: 8px;
    background: var(--bg-surface);
    overflow: hidden;
  }

  .app-item {
    display: flex;
    flex-direction: column;
    align-items: flex-start !important;
    gap: 2px;
    padding: 12px 20px !important;
    border: none !important;
    border-bottom: 1px solid var(--border) !important;
    border-radius: 0 !important;
    background: transparent !important;
    width: 100%;
    text-align: left;
    transition: all 0.15s !important;
    box-shadow: none !important;
    transform: none !important;
    margin: 0 !important;
  }

  .app-item:hover {
    background: var(--bg-active) !important;
  }

  .app-name {
    color: var(--primary);
    font-family: var(--font-mono);
    font-weight: bold;
    font-size: 13px;
  }

  .app-title {
    color: var(--text-muted);
    font-size: 11px;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
    width: 100%;
  }

  .loading {
    padding: 40px;
    text-align: center;
    color: var(--text-muted);
    font-family: var(--font-mono);
    text-transform: uppercase;
    letter-spacing: 2px;
    animation: pulse 1.5s infinite;
  }

  .no-apps-found {
    padding: 40px;
    text-align: center;
    color: var(--text-muted);
    font-size: 13px;
    font-style: italic;
    background: var(--bg-surface);
  }

  @keyframes pulse {
    0% { opacity: 0.5; }
    50% { opacity: 1; }
    100% { opacity: 0.5; }
  }
</style>