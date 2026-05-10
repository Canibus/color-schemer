<!-- src/lib/ProfileEditor.svelte -->
<script lang="ts">
  import type { DisplayProfile, DisplaySettings } from "./types";
  import { i18n } from "./i18n.svelte";
  
  let { 
    profile, 
    onSave, 
    onCancel, 
    onPreview, 
    onDelete = undefined 
  } = $props<{
    profile: DisplayProfile;
    onSave: (updated: DisplayProfile) => void;
    onCancel: () => void;
    onPreview: (settings: DisplaySettings) => void;
    onDelete?: () => void;
  }>();

  // Local copy for editing
  let edited = $state(JSON.parse(JSON.stringify(profile)) as DisplayProfile);

  $effect(() => {
    edited = JSON.parse(JSON.stringify(profile)) as DisplayProfile;
  });

  function handleInput() {
      onPreview(edited.settings);
  }
</script>

<div class="editor">
  <h3>{i18n.t('editor.title')}</h3>
  
  <label>{i18n.t('editor.name')}
      <input type="text" bind:value={edited.name} />
  </label>
  <label>{i18n.t('editor.description')}
      <input type="text" bind:value={edited.description} />
  </label>

  <div class="field-group">
      <div class="group-label">{i18n.t('editor.settings')}</div>
      <div class="group-content">
          <label>{i18n.t('editor.brightness')} (<span class="value-display">{edited.settings.brightness}</span>)
              <input type="range" min="0" max="2" step="0.01" bind:value={edited.settings.brightness} oninput={handleInput} />
          </label>
          <label>{i18n.t('editor.contrast')} (<span class="value-display">{edited.settings.contrast}</span>)
              <input type="range" min="0" max="2" step="0.01" bind:value={edited.settings.contrast} oninput={handleInput} />
          </label>
          <label>{i18n.t('editor.gamma')} (<span class="value-display">{edited.settings.gamma}</span>)
              <input type="range" min="0.1" max="3" step="0.01" bind:value={edited.settings.gamma} oninput={handleInput} />
          </label>
          <label>{i18n.t('editor.vibrance')} (<span class="value-display">{edited.settings.digital_vibrance}</span>)
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
  h3 { color: var(--primary); text-transform: uppercase; font-size: 16px; margin-top: 0; }
  
  label { display: flex; flex-direction: column; gap: 5px; }
  
  input[type="range"] {
    accent-color: var(--primary);
    cursor: pointer;
  }
  
  .value-display {
    font-family: var(--font-mono);
    color: var(--primary);
    font-weight: bold;
  }
  
  .field-group {
    background: var(--bg-layer);
    border: 1px solid var(--border);
    border-radius: 8px;
    margin-bottom: 10px;
    overflow: hidden;
  }
  
  .group-label {
    background: var(--bg-active);
    color: var(--primary);
    font-family: var(--font-mono);
    font-size: 11px;
    text-transform: uppercase;
    padding: 6px 12px;
    border-bottom: 1px solid var(--border);
    letter-spacing: 0.5px;
  }

  .group-content {
    padding: 18px;
    display: flex;
    flex-direction: column;
    gap: 15px;
  }
  
  input[type="text"] {
    background: var(--bg-surface);
    border: 1px solid var(--border);
    color: var(--text-main);
    padding: 8px;
    border-radius: 6px;
  }
  
  input[type="text"]:focus {
    border-color: var(--primary);
    outline: none;
    box-shadow: 0 0 5px var(--primary-glow);
  }
  
  .actions { display: flex; justify-content: flex-end; gap: 10px; margin-top: 10px; }
  
  .delete-btn { 
    background: var(--error-dim); 
    border: none; 
    color: var(--text-main); 
    border-radius: 8px; 
    padding: 8px 16px;
    margin-right: auto;
    cursor: pointer;
  }
  
  .delete-btn:hover { 
    background: var(--error); 
    box-shadow: 0 0 10px var(--error-glow); 
  }

  button:not(.delete-btn) {
    background: var(--bg-active);
    border: 1px solid var(--border);
    color: var(--text-main);
    padding: 8px 16px;
    border-radius: 8px;
    cursor: pointer;
  }

  button:not(.delete-btn):hover {
    border-color: var(--primary);
    background: var(--bg-layer);
  }
</style>
