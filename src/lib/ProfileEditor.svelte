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