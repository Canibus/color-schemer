<!-- src/lib/ProfileEditor.svelte -->
<script lang="ts">
  import type { DisplayInfo, DisplayProfile, DisplaySettings } from "./types";
  import { i18n } from "./i18n.svelte";
  
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
  let edited = $state(JSON.parse(JSON.stringify(profile)) as DisplayProfile);

  $effect(() => {
    // We update 'edited' only when the 'profile' prop changes (e.g. switching which profile is being edited)
    let copy = JSON.parse(JSON.stringify(profile)) as DisplayProfile;
    if (!copy.target_displays) {
        copy.target_displays = [];
    }
    edited = copy;
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
      // Note: We don't call handleInput() here to avoid applying the profile 
      // just by clicking a display checkbox. Preview only happens on slider changes.
  }
</script>

<div class="editor">
  <h3>{i18n.t('editor.title')}</h3>
  
  <div class="field-group">
    <div class="group-label">{i18n.t('editor.info')}</div>
    <div class="group-content">
      <label>{i18n.t('editor.name')}
          <input type="text" bind:value={edited.name} />
      </label>
      <label>{i18n.t('editor.description')}
          <input type="text" bind:value={edited.description} />
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
</style>