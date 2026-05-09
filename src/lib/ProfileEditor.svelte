<!-- src/lib/ProfileEditor.svelte -->
<script lang="ts">
  import type { DisplayProfile, DisplaySettings } from "./types";
  export let profile: DisplayProfile;
  export let onSave: (updated: DisplayProfile) => void;
  export let onCancel: () => void;
  export let onPreview: (settings: DisplaySettings) => void;
  export let onDelete: (() => void) | undefined = undefined;

  // Local copy for editing
  let edited = JSON.parse(JSON.stringify(profile)) as DisplayProfile;

  function handleInput() {
      onPreview(edited.settings);
  }
</script>

<div class="editor">
  <h3>Edit Profile</h3>
  
  <label>Name
      <input type="text" bind:value={edited.name} />
  </label>
  <label>Description
      <input type="text" bind:value={edited.description} />
  </label>

  <fieldset>
      <legend>Settings</legend>
      <label>Brightness ({edited.settings.brightness})
          <input type="range" min="0" max="2" step="0.01" bind:value={edited.settings.brightness} on:input={handleInput} />
      </label>
      <label>Contrast ({edited.settings.contrast})
          <input type="range" min="0" max="2" step="0.01" bind:value={edited.settings.contrast} on:input={handleInput} />
      </label>
      <label>Gamma ({edited.settings.gamma})
          <input type="range" min="0.1" max="3" step="0.01" bind:value={edited.settings.gamma} on:input={handleInput} />
      </label>
      <label>Digital Vibrance ({edited.settings.digital_vibrance})
          <input type="range" min="-50" max="100" step="1" bind:value={edited.settings.digital_vibrance} on:input={handleInput} />
      </label>
  </fieldset>

  <div class="actions">
      {#if onDelete}
        <button class="delete-btn" on:click={onDelete}>Delete</button>
      {/if}
      <button on:click={onCancel}>Cancel</button>
      <button on:click={() => onSave(edited)}>Save</button>
  </div>
</div>

<style>
  .editor { display: flex; flex-direction: column; gap: 15px; }
  label { display: flex; flex-direction: column; gap: 5px; }
  fieldset { display: flex; flex-direction: column; gap: 10px; border: 1px solid #2a3440; border-radius: 5px; padding: 15px; }
  .actions { display: flex; justify-content: flex-end; gap: 10px; margin-top: 10px; }
  input[type="text"] { background: #121821; border: 1px solid #2a3440; color: inherit; padding: 5px; border-radius: 3px; }
  .delete-btn { background: #7f1d1d; margin-right: auto; }
  .delete-btn:hover { background: #991b1b; }
</style>
