<!-- src/App.svelte -->
<script lang="ts">
  import { onMount } from "svelte";
  import { invoke } from "@tauri-apps/api/tauri";
  import type { DisplayProfile, AppConfig } from "./lib/types";
  import ProfileList from "./lib/ProfileList.svelte";
  import ProfileEditor from "./lib/ProfileEditor.svelte";
  import SettingsTab from "./lib/SettingsTab.svelte";

  let profiles: DisplayProfile[] = [];
  let activeIndex: number | null = null;
  let config: AppConfig | null = null;
  
  let activeTab: 'profiles' | 'settings' = 'profiles';
  let editingIndex: number | null = null;
  let status = "Loading…";

  async function loadState() {
    status = "Loading state...";
    try {
        const res = await invoke<{ profiles: DisplayProfile[]; active_index: number }>("get_profiles_state");
        profiles = res.profiles;
        activeIndex = res.active_index;
        config = await invoke<AppConfig>("get_config");
        status = "Ready";
    } catch(e) {
        status = `Error: ${e}`;
    }
  }

  onMount(loadState);

  async function applyProfile(index: number) {
      status = "Applying...";
      try {
          await invoke("apply_profile", { index });
          await loadState();
      } catch (e) {
          status = `Error: ${e}`;
      }
  }
</script>

<main class="container">
  <header class="header">
    <h1>color-schemer</h1>
    <div class="status">{status}</div>
  </header>

  <nav class="tabs">
      <button class:active={activeTab === 'profiles'} on:click={() => { activeTab = 'profiles'; editingIndex = null; }}>Profiles</button>
      <button class:active={activeTab === 'settings'} on:click={() => activeTab = 'settings'}>Settings</button>
  </nav>

  <div class="content">
      {#if activeTab === 'profiles'}
          {#if editingIndex !== null && config}
              <ProfileEditor 
                  profile={config.profiles[editingIndex]} 
                  onSave={(p) => { /* TODO */ }} 
                  onCancel={() => editingIndex = null} 
                  onPreview={(s) => { /* TODO */ }} 
              />
          {:else}
              <ProfileList 
                  {profiles} 
                  {activeIndex} 
                  onApply={applyProfile} 
                  onEdit={(i) => editingIndex = i} 
              />
          {/if}
      {:else if activeTab === 'settings' && config}
          <SettingsTab {config} onSave={(c) => { /* TODO */ }} />
      {/if}
  </div>
</main>

<style>
  :global(html, body) {
    height: 100%;
  }
  :global(body) {
    margin: 0;
    font-family: system-ui, -apple-system, Segoe UI, Roboto, Ubuntu, Cantarell, Noto Sans, Arial,
      sans-serif;
    background: #0b0d10;
    color: #e8eef6;
  }

  .container {
    padding: 18px;
    display: grid;
    gap: 14px;
  }

  .header {
    display: flex;
    align-items: baseline;
    justify-content: space-between;
    gap: 12px;
  }
  h1 {
    margin: 0;
    font-size: 18px;
    letter-spacing: 0.2px;
  }
  .status {
    font-size: 12px;
    opacity: 0.85;
  }

  /* Basic tab styling */
  .tabs { display: flex; gap: 10px; margin-bottom: 20px; }
  .tabs button {
    appearance: none;
    border: 1px solid #2a3440;
    background: #121821;
    color: inherit;
    padding: 8px 16px;
    border-radius: 8px;
    cursor: pointer;
  }
  .tabs button.active { font-weight: bold; border-color: #6fb1ff; background: #1a232e; }

  button {
    appearance: none;
    border: 1px solid #2a3440;
    background: #121821;
    color: inherit;
    padding: 10px 12px;
    border-radius: 10px;
    cursor: pointer;
    text-align: left;
  }
  button:hover:enabled {
    border-color: #3a4b5d;
    background: #141c26;
  }
  button:disabled {
    opacity: 0.55;
    cursor: not-allowed;
  }
</style>
