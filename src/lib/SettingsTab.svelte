<!-- src/lib/SettingsTab.svelte -->
<script lang="ts">
  import { invoke } from "@tauri-apps/api/tauri";
  import type { AppConfig, HotkeyConfig, GpuInfo } from "./types";
  import { i18n } from "./i18n.svelte";
  import { untrack } from "svelte";
  
  let { 
    config,
    gpuInfo,
    onSave,
    onReset
  } = $props<{
    config: AppConfig;
    gpuInfo: GpuInfo | null;
    onSave: (updated: AppConfig) => void;
    onReset: () => void;
  }>();

  let edited = $state(untrack(() => $state.snapshot(config)));
  let recordingKey = $state<keyof HotkeyConfig | null>(null);

  $effect(() => {
    // We update 'edited' only when the 'config' prop changes from outside
    edited = $state.snapshot(config);
  });

  function handleLanguageChange() {
      i18n.setLanguage(edited.language);
      onSave(edited);
  }

  async function startRecording(key: keyof HotkeyConfig) {
      recordingKey = key;
      await invoke("set_recording_mode", { active: true });
  }

  function normalizeHotkey(str: string): string {
      if (!str) return "";
      const parts = str.split("+").map(s => {
          let t = s.trim().toUpperCase();
          if (t === "CONTROL" || t === "CTRL") return "CTRL";
          if (t === "META" || t === "WIN" || t === "SUPER") return "WIN";
          return t;
      });
      const key = parts.pop() || "";
      const mods = parts.sort();
      return mods.length > 0 ? `${mods.join("+")}+${key}` : key;
  }

  async function handleKeyDown(event: KeyboardEvent) {
      if (!recordingKey) return;

      // Prevent default behavior (e.g., Ctrl+S saving the page)
      event.preventDefault();

      if (event.key === "Escape") {
          recordingKey = null;
          await invoke("set_recording_mode", { active: false });
          return;
      }

      // Ignore if only a modifier key is pressed
      const isModifier = ["Control", "Shift", "Alt", "Meta"].includes(event.key);
      if (isModifier) return;

      const mods: string[] = [];
      if (event.ctrlKey) mods.push("Ctrl");
      if (event.shiftKey) mods.push("Shift");
      if (event.altKey) mods.push("Alt");
      if (event.metaKey) mods.push("Win");

      let mainKey = event.key.toUpperCase();
      
      // Map common key names to what our Rust backend expects
      if (mainKey.startsWith("F") && mainKey.length > 1) {
          // F1, F2, etc. are already fine
      } else if (mainKey === " ") {
          mainKey = "Space";
      }

      const hotkeyStr = mods.length > 0 ? `${mods.join("+")}+${mainKey}` : mainKey;
      const normalizedNew = normalizeHotkey(hotkeyStr);
      
      // Create a copy to ensure Svelte 5 reactivity triggers properly on assignment
      const newHotkeys = { ...edited.hotkeys };
      
      // Clear this hotkey if it's already used by another action
      const keys = Object.keys(newHotkeys) as (keyof HotkeyConfig)[];
      keys.forEach((key) => {
          if (key !== recordingKey && normalizeHotkey(newHotkeys[key]) === normalizedNew) {
              newHotkeys[key] = "";
          }
      });

      newHotkeys[recordingKey] = hotkeyStr;
      edited.hotkeys = newHotkeys;
      recordingKey = null;
      await invoke("set_recording_mode", { active: false });

      // Auto-save the new hotkey configuration
      onSave(edited);
  }
</script>

<svelte:window onkeydown={handleKeyDown} />

<div class="settings">
  <div class="field-group">
      <div class="group-label">{i18n.t('settings.language')}</div>
      <div class="group-content">
          <div class="select-wrapper">
              <select bind:value={edited.language} onchange={handleLanguageChange}>
                  <option value="en">ENGLISH [EN]</option>
                  <option value="ru">РУССКИЙ [RU]</option>
              </select>
              <div class="select-arrow"></div>
          </div>
      </div>
  </div>

  <div class="field-group">
      <div class="group-label">{i18n.t('settings.hotkeys')}</div>
      <div class="group-content">
          <div class="hotkey-item">
              <span>{i18n.t('hotkey.next')}</span>
              <button 
                  class:recording={recordingKey === 'next_profile'} 
                  class:empty={!edited.hotkeys.next_profile && recordingKey !== 'next_profile'}
                  onclick={() => startRecording('next_profile')}
              >
                  {#if recordingKey === 'next_profile'}
                      {i18n.t('hotkey.recording')}
                  {:else}
                      {edited.hotkeys.next_profile || i18n.t('hotkey.none')}
                  {/if}
              </button>
          </div>
          <div class="hotkey-item">
              <span>{i18n.t('hotkey.prev')}</span>
              <button 
                  class:recording={recordingKey === 'prev_profile'} 
                  class:empty={!edited.hotkeys.prev_profile && recordingKey !== 'prev_profile'}
                  onclick={() => startRecording('prev_profile')}
              >
                  {#if recordingKey === 'prev_profile'}
                      {i18n.t('hotkey.recording')}
                  {:else}
                      {edited.hotkeys.prev_profile || i18n.t('hotkey.none')}
                  {/if}
              </button>
          </div>
          <div class="hotkey-item">
              <span>{i18n.t('hotkey.reset')}</span>
              <button 
                  class:recording={recordingKey === 'reset'} 
                  class:empty={!edited.hotkeys.reset && recordingKey !== 'reset'}
                  onclick={() => startRecording('reset')}
              >
                  {#if recordingKey === 'reset'}
                      {i18n.t('hotkey.recording')}
                  {:else}
                      {edited.hotkeys.reset || i18n.t('hotkey.none')}
                  {/if}
              </button>
          </div>
      </div>
  </div>

  <div class="field-group">
      <div class="group-label">{i18n.t('settings.behavior')}</div>
      <div class="group-content">
          <label class="checkbox-label">
              <input type="checkbox" bind:checked={edited.auto_start} onchange={() => onSave(edited)} />
              {i18n.t('settings.autostart')}
          </label>
          <label class="checkbox-label">
              <input type="checkbox" bind:checked={edited.start_minimized} onchange={() => onSave(edited)} />
              {i18n.t('settings.minimized')}
          </label>
      </div>
  </div>

  <div class="field-group">
    <div class="group-label">{i18n.t('settings.hardware')}</div>
    <div class="group-content hardware-info">
        <div class="info-row">
            <span>{i18n.t('settings.gpu')}:</span>
            <span class="value">{gpuInfo?.name || "..."}</span>
        </div>
        <div class="info-row">
            <span>{i18n.t('settings.nvapi_status')}:</span>
            <span class="value" class:mock={gpuInfo?.is_mock}>
                {gpuInfo?.is_mock ? i18n.t('settings.status_mock') : i18n.t('settings.status_ok')}
            </span>
        </div>
    </div>
  </div>

  <div class="field-group">
    <div class="group-label">{i18n.t('settings.maintenance')}</div>
    <div class="group-content">
        <button class="reset-btn" onclick={onReset}>
            {i18n.t('settings.reset_defaults')}
        </button>
    </div>
  </div>
</div>

<style>
  .settings { 
    color: var(--text-main); 
    display: flex; 
    flex-direction: column; 
    gap: 15px; 
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
    gap: 15px;
  }

  .select-wrapper {
    position: relative;
    width: 100%;
  }

  select {
    appearance: none;
    width: 100%;
    background: var(--bg-surface);
    border: 1px solid var(--primary-dim);
    color: var(--primary);
    font-family: var(--font-mono);
    padding: 10px 16px;
    border-radius: 6px;
    cursor: pointer;
    font-size: 13px;
    transition: all 0.2s;
    box-shadow: inset 0 -2px 0 var(--primary-dim);
    outline: none;
  }

  select:hover {
    border-color: var(--primary);
    background: var(--bg-active);
  }

  select:focus {
    border-color: var(--primary);
    box-shadow: var(--glow-shadow), inset 0 0 5px var(--primary-glow);
  }

  .select-arrow {
    position: absolute;
    right: 12px;
    top: 50%;
    transform: translateY(-50%);
    width: 0;
    height: 0;
    border-left: 5px solid transparent;
    border-right: 5px solid transparent;
    border-top: 6px solid var(--primary);
    pointer-events: none;
  }

  option {
    background: var(--bg-layer);
    color: var(--text-main);
  }
  
  .hotkey-item { 
    display: flex; 
    align-items: center; 
    justify-content: space-between; 
    gap: 10px; 
  }
  .hotkey-item span { 
    font-size: 13px; 
    font-family: var(--font-mono);
  }
  .hotkey-item button {
    background: var(--bg-surface);
    border: 1px solid var(--primary-dim);
    color: var(--primary);
    font-family: var(--font-mono);
    padding: 8px 16px;
    border-radius: 6px;
    cursor: pointer;
    min-width: 140px;
    text-align: center;
    font-size: 13px;
    transition: all 0.2s;
    box-shadow: inset 0 -2px 0 var(--primary-dim); /* "Keycap" look */
  }
  .hotkey-item button:hover {
    border-color: var(--primary);
    background: var(--bg-active);
  }
  .hotkey-item button.recording {
    border-color: var(--primary);
    box-shadow: var(--glow-shadow), inset 0 0 5px var(--primary-glow);
    color: var(--text-main);
  }
  .hotkey-item button.empty {
    color: var(--text-muted);
    border-color: var(--border);
    opacity: 0.7;
    font-style: italic;
    box-shadow: none;
  }
  .checkbox-label { 
    display: flex; 
    align-items: center; 
    gap: 12px; 
    font-size: 13px; 
    cursor: pointer; 
    font-family: var(--font-mono);
    color: var(--text-main);
    user-select: none;
    transition: all 0.2s;
  }
  .checkbox-label:hover {
    color: var(--primary);
  }
  
  /* Custom Toggle Styling */
  .checkbox-label input {
    appearance: none;
    width: 32px;
    height: 18px;
    background: var(--bg-surface);
    border: 1px solid var(--primary-dim);
    border-radius: 10px;
    position: relative;
    cursor: pointer;
    transition: all 0.2s;
    box-shadow: inset 0 2px 5px rgba(0,0,0,0.5);
  }

  .checkbox-label input:checked {
    background: var(--primary-dim);
    border-color: var(--primary);
    box-shadow: 0 0 10px var(--primary-glow), inset 0 2px 5px rgba(0,0,0,0.2);
  }

  .checkbox-label input::before {
    content: '';
    position: absolute;
    width: 10px;
    height: 10px;
    background: var(--text-muted);
    border-radius: 50%;
    top: 3px;
    left: 4px;
    transition: all 0.2s cubic-bezier(0.34, 1.56, 0.64, 1);
  }

  .checkbox-label input:checked::before {
    background: var(--primary);
    transform: translateX(12px);
    box-shadow: 0 0 5px var(--primary-glow);
  }

  .hardware-info {
      gap: 10px;
  }

  .info-row {
      display: flex;
      justify-content: space-between;
      font-size: 13px;
      font-family: var(--font-mono);
  }

  .info-row .value {
      color: var(--primary);
  }

  .info-row .value.mock {
      color: var(--error);
  }

  .reset-btn {
      appearance: none;
      background: var(--error-dim);
      border: 1px solid var(--error);
      color: var(--text-main);
      padding: 10px;
      border-radius: 8px;
      cursor: pointer;
      text-align: center;
      font-family: var(--font-mono);
      font-size: 12px;
      transition: all 0.2s;
      text-transform: uppercase;
  }

  .reset-btn:hover {
      background: var(--error);
      box-shadow: 0 0 10px var(--error-glow);
  }
</style>
