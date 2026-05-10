<!-- src/App.svelte -->
<script lang="ts">
  import { onMount } from "svelte";
  import "./theme.css";
  import { invoke } from "@tauri-apps/api/tauri";
  import type { DisplayProfile, AppConfig, DisplaySettings } from "./lib/types";
  import ProfileList from "./lib/ProfileList.svelte";
  import ProfileEditor from "./lib/ProfileEditor.svelte";
  import SettingsTab from "./lib/SettingsTab.svelte";
  import { i18n } from "./lib/i18n.svelte";

  let profiles = $state<DisplayProfile[]>([]);
  let activeIndex = $state<number | null>(null);
  let config = $state<AppConfig | null>(null);
  
  let activeTab = $state<'profiles' | 'settings'>('profiles');
  let editingIndex = $state<number | null>(null);
  let status = $state("");

  async function loadState() {
    status = i18n.t("app.status.loading");
    try {
        const res = await invoke<{ profiles: DisplayProfile[]; active_index: number }>("get_profiles_state");
        profiles = res.profiles;
        activeIndex = res.active_index;
        config = await invoke<AppConfig>("get_config");
        if (config && config.language) {
            i18n.setLanguage(config.language);
        }
        status = i18n.t("app.status.ready");
    } catch(e) {
        status = `${i18n.t("app.status.error")}: ${e}`;
    }
  }

  onMount(loadState);

  async function applyProfile(index: number) {
      status = i18n.t("app.status.applying");
      try {
          await invoke("apply_profile", { index });
          await loadState();
      } catch (e) {
          status = `${i18n.t("app.status.error")}: ${e}`;
      }
  }

  async function previewSettings(settings: DisplaySettings) {
      try {
          await invoke("preview_settings", { settings });
      } catch (e) {
          console.error("Preview failed:", e);
      }
  }

  async function saveProfile(updatedProfile: DisplayProfile) {
      if (editingIndex === null || !config) return;
      status = i18n.t("app.status.saving");
      try {
          // Update local config object
          const newConfig = { ...config };
          newConfig.profiles[editingIndex] = updatedProfile;
          
          // Save to backend
          await invoke("save_config", { updated: newConfig });
          
          // If we edited the currently active profile, apply it properly to update backend state
          if (editingIndex === activeIndex) {
              await invoke("apply_profile", { index: editingIndex });
          }
          
          await loadState();
          editingIndex = null;
          status = i18n.t("app.status.saved");
      } catch (e) {
          status = `${i18n.t("app.status.error")}: ${e}`;
      }
  }

  async function deleteProfile() {
      if (editingIndex === null || !config) return;
      if (!confirm(i18n.t("app.confirm.delete"))) return;
      
      status = i18n.t("app.status.deleting");
      try {
          const newConfig = { ...config };
          newConfig.profiles.splice(editingIndex, 1);
          await invoke("save_config", { updated: newConfig });
          
          // If we deleted the active profile, reset to index 0 (if available)
          if (editingIndex === activeIndex && newConfig.profiles.length > 0) {
              await invoke("apply_profile", { index: 0 });
          }
          
          await loadState();
          editingIndex = null;
          status = i18n.t("app.status.deleted");
      } catch (e) {
          status = `${i18n.t("app.status.error")}: ${e}`;
      }
  }

  async function saveSettings(updatedConfig: AppConfig) {
      status = i18n.t("app.status.saving");
      try {
          await invoke("save_config", { updated: updatedConfig });
          await loadState();
          status = i18n.t("app.status.saved");
      } catch (e) {
          status = `${i18n.t("app.status.error")}: ${e}`;
      }
  }
  
  async function cancelEdit() {
      editingIndex = null;
      // Re-apply the active profile to revert any live preview changes
      if (activeIndex !== null) {
          try {
              await invoke("apply_profile", { index: activeIndex });
          } catch(e) {
              console.error("Failed to restore active profile", e);
          }
      }
  }
</script>

<main class="container">
  <header class="header">
    <div class="brand">
      <img src="/icon.svg" alt="" class="logo" />
      <h1>color-schemer</h1>
    </div>
    <div class="status">{status}</div>
  </header>

  <nav class="tabs">
      <button class:active={activeTab === 'profiles'} onclick={() => { activeTab = 'profiles'; editingIndex = null; }}>{i18n.t('nav.profiles')}</button>
      <button class:active={activeTab === 'settings'} onclick={() => activeTab = 'settings'}>{i18n.t('nav.settings')}</button>
  </nav>

  <div class="content">
      {#if activeTab === 'profiles'}
          {#if editingIndex !== null && config}
              {#if editingIndex === -1}
                  <ProfileEditor 
                      profile={{ name: i18n.t('editor.new_profile'), description: "", settings: { brightness: 1, contrast: 1, gamma: 1, digital_vibrance: 0 } }} 
                      onSave={(p) => {
                          const newConfig = { ...config };
                          newConfig.profiles.push(p);
                          saveSettings(newConfig).then(() => editingIndex = null);
                      }} 
                      onCancel={cancelEdit} 
                      onPreview={previewSettings} 
                  />
              {:else}
                  <ProfileEditor 
                      profile={config.profiles[editingIndex]} 
                      onSave={saveProfile} 
                      onCancel={cancelEdit} 
                      onPreview={previewSettings} 
                      onDelete={deleteProfile}
                  />
              {/if}
          {:else}
              <ProfileList 
                  {profiles} 
                  {activeIndex} 
                  onApply={applyProfile} 
                  onEdit={(i) => editingIndex = i} 
              />
          {/if}
      {:else if activeTab === 'settings' && config}
          <SettingsTab {config} onSave={saveSettings} />
      {/if}
  </div>
</main>

<style>
  :global(html, body) {
    height: 100%;
  }
  :global(body) {
    margin: 0;
    font-family: var(--font-sans);
  }

  .container {
    padding: 18px;
    display: grid;
    gap: 14px;
    user-select: none;
  }

  .header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 12px;
    padding-bottom: 5px;
  }
  .brand {
    display: flex;
    align-items: center;
    gap: 10px;
  }
  .logo {
    width: 24px;
    height: 24px;
    filter: drop-shadow(0 0 5px var(--primary-glow));
  }
  h1 {
    margin: 0;
    font-size: 18px;
    color: var(--primary);
    text-transform: uppercase;
    letter-spacing: 1px;
    font-family: var(--font-mono);
  }
  .status {
    font-size: 11px;
    color: var(--primary);
    opacity: 0.8;
    font-family: var(--font-mono);
    text-transform: uppercase;
  }

  /* Basic tab styling */
  .tabs { display: flex; gap: 10px; margin-bottom: 20px; }
  .tabs button {
    appearance: none;
    background: var(--bg-layer);
    border: 1px solid var(--border);
    color: var(--text-muted);
    padding: 8px 16px;
    border-radius: 8px;
    cursor: pointer;
    transition: all 0.2s ease-out;
    text-align: center;
    font-family: var(--font-mono);
    text-transform: uppercase;
    font-size: 12px;
    letter-spacing: 1px;
  }
  .tabs button.active {
    color: var(--primary);
    border-color: var(--primary);
    box-shadow: var(--glow-shadow);
    background: var(--bg-active);
  }

  button {
    appearance: none;
    border: 1px solid var(--border);
    background: var(--bg-layer);
    color: inherit;
    padding: 10px 12px;
    border-radius: 10px;
    cursor: pointer;
    text-align: left;
    transition: all 0.15s ease-out;
    font-family: var(--font-sans);
  }
  button:hover:enabled {
    border-color: var(--primary);
    background: var(--bg-active);
  }
  button:disabled {
    opacity: 0.55;
    cursor: not-allowed;
  }
</style>
