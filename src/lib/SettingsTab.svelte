<!-- src/lib/SettingsTab.svelte -->
<script lang="ts">
  import type { AppConfig } from "./types";
  export let config: AppConfig;
  export let onSave: (updated: AppConfig) => void;

  let edited = JSON.parse(JSON.stringify(config)) as AppConfig;
</script>

<div class="settings">
  <h3>Global Settings</h3>
  
  <fieldset>
      <legend>Hotkeys</legend>
      <label>Next Profile
          <input type="text" bind:value={edited.hotkeys.next_profile} />
      </label>
      <label>Previous Profile
          <input type="text" bind:value={edited.hotkeys.prev_profile} />
      </label>
      <label>Reset Profile
          <input type="text" bind:value={edited.hotkeys.reset} />
      </label>
  </fieldset>

  <fieldset>
      <legend>Behavior</legend>
      <label class="checkbox-label">
          <input type="checkbox" bind:checked={edited.show_notifications} />
          Show Notifications
      </label>
      <label class="checkbox-label">
          <input type="checkbox" bind:checked={edited.start_minimized} />
          Start Minimized
      </label>
  </fieldset>

  <div class="actions">
      <button on:click={() => onSave(edited)}>Save Settings</button>
  </div>
</div>

<style>
  .settings { display: flex; flex-direction: column; gap: 15px; }
  label { display: flex; flex-direction: column; gap: 5px; }
  .checkbox-label { flex-direction: row; align-items: center; }
  fieldset { display: flex; flex-direction: column; gap: 10px; border: 1px solid #2a3440; border-radius: 5px; padding: 15px; }
  .actions { display: flex; justify-content: flex-end; margin-top: 10px; }
  input[type="text"] { background: #121821; border: 1px solid #2a3440; color: inherit; padding: 5px; border-radius: 3px; }
</style>
