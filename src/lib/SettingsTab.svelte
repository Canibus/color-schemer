<!-- src/lib/SettingsTab.svelte -->
<script lang="ts">
  import type { AppConfig, HotkeyConfig } from "./types";
  
  let { 
    config, 
    onSave 
  } = $props<{
    config: AppConfig;
    onSave: (updated: AppConfig) => void;
  }>();

  let edited = $state(JSON.parse(JSON.stringify(config)) as AppConfig);
  let recordingKey = $state<keyof HotkeyConfig | null>(null);

  $effect(() => {
    edited = JSON.parse(JSON.stringify(config)) as AppConfig;
  });

  function startRecording(key: keyof HotkeyConfig) {
      recordingKey = key;
  }

  function handleKeyDown(event: KeyboardEvent) {
      if (!recordingKey) return;

      // Prevent default behavior (e.g., Ctrl+S saving the page)
      event.preventDefault();

      if (event.key === "Escape") {
          recordingKey = null;
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
          mainKey = "Space"; // Though space isn't explicitly in our Rust match yet, good for future
      } else if (mainKey.length > 1) {
          // Other special keys might need mapping, but our backend mostly wants single chars or Fn keys
      }

      const hotkeyStr = mods.length > 0 ? `${mods.join("+")}+${mainKey}` : mainKey;
      
      edited.hotkeys[recordingKey] = hotkeyStr;
      recordingKey = null;
  }
</script>

<svelte:window onkeydown={handleKeyDown} />

<div class="settings">
  <h3>Global Settings</h3>
  
  <div class="field-group">
      <div class="group-label">Hotkeys</div>
      <div class="group-content">
          <div class="hotkey-item">
              <span>Next Profile</span>
              <button 
                  class:recording={recordingKey === 'next_profile'} 
                  onclick={() => startRecording('next_profile')}
              >
                  {recordingKey === 'next_profile' ? "Press keys... (Esc to cancel)" : edited.hotkeys.next_profile}
              </button>
          </div>
          <div class="hotkey-item">
              <span>Previous Profile</span>
              <button 
                  class:recording={recordingKey === 'prev_profile'} 
                  onclick={() => startRecording('prev_profile')}
              >
                  {recordingKey === 'prev_profile' ? "Press keys... (Esc to cancel)" : edited.hotkeys.prev_profile}
              </button>
          </div>
          <div class="hotkey-item">
              <span>Reset Profile</span>
              <button 
                  class:recording={recordingKey === 'reset'} 
                  onclick={() => startRecording('reset')}
              >
                  {recordingKey === 'reset' ? "Press keys... (Esc to cancel)" : edited.hotkeys.reset}
              </button>
          </div>
      </div>
  </div>

  <div class="field-group">
      <div class="group-label">Behavior</div>
      <div class="group-content">
          <label class="checkbox-label">
              <input type="checkbox" bind:checked={edited.show_notifications} />
              Show Notifications
          </label>
          <label class="checkbox-label">
              <input type="checkbox" bind:checked={edited.start_minimized} />
              Start Minimized
          </label>
      </div>
  </div>
  <div class="actions">
      <button class="save-btn" onclick={() => onSave(edited)}>Save Settings</button>
  </div>
</div>

<style>
  .settings { 
    color: var(--text-main); 
    display: flex; 
    flex-direction: column; 
    gap: 15px; 
  }
  h3 { 
    color: var(--primary); 
    text-transform: uppercase; 
    font-size: 16px; 
    margin-top: 0; 
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
  
  .hotkey-item { 
    display: flex; 
    align-items: center; 
    justify-content: space-between; 
    gap: 10px; 
  }
  .hotkey-item span { 
    font-size: 14px; 
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
  .checkbox-label { 
    display: flex; 
    align-items: center; 
    gap: 10px; 
    font-size: 14px; 
    cursor: pointer; 
  }
  .actions { 
    display: flex; 
    justify-content: flex-end; 
    margin-top: 10px; 
  }
  .save-btn {
    background: var(--primary);
    color: var(--bg-surface);
    border: none;
    padding: 10px 24px;
    border-radius: 8px;
    font-weight: bold;
    cursor: pointer;
    transition: all 0.2s;
  }
  .save-btn:hover {
    filter: brightness(1.1);
    box-shadow: var(--glow-shadow);
  }
</style>
