<!-- src/App.svelte -->
<script lang="ts">
  import { onMount } from "svelte";
  import "./theme.css";
  import { invoke } from "@tauri-apps/api/tauri";
  import { listen } from "@tauri-apps/api/event";
  import { appWindow } from "@tauri-apps/api/window";
  import type { DisplayProfile, AppConfig, DisplaySettings, DisplayInfo, GpuInfo } from "./lib/types";
  import ProfileList from "./lib/ProfileList.svelte";
  import ProfileEditor from "./lib/ProfileEditor.svelte";
  import SettingsTab from "./lib/SettingsTab.svelte";
  import { i18n } from "./lib/i18n.svelte";

  let profiles = $state<DisplayProfile[]>([]);
  let activeIndex = $state<number | null>(null);
  let config = $state<AppConfig | null>(null);
  let displays = $state<DisplayInfo[]>([]);
  let gpuInfo = $state<GpuInfo | null>(null);
  
  let activeTab = $state<'profiles' | 'settings'>('profiles');
  let editingIndex = $state<number | null>(null);
  let status = $state("");

  function minimizeWindow() {
    appWindow.minimize();
  }

  function closeWindow() {
    // For tray apps, "close" usually means hide to tray
    appWindow.hide();
  }

  function startDragging(e: MouseEvent) {
    // Only drag if we didn't click a button or interactive element
    if (e.button === 0 && !(e.target as HTMLElement).closest('button')) {
      appWindow.startDragging();
    }
  }

  async function loadState() {
    status = i18n.t("app.status.loading");
    try {
        const [res, cfg, ds, gi] = await Promise.all([
            invoke<{ profiles: DisplayProfile[]; active_index: number }>("get_profiles_state"),
            invoke<AppConfig>("get_config"),
            invoke<DisplayInfo[]>("get_displays"),
            invoke<GpuInfo>("get_gpu_info")
        ]);
        
        profiles = res.profiles;
        activeIndex = res.active_index;
        config = cfg;
        displays = ds;
        gpuInfo = gi;
        
        if (config && config.language) {
            i18n.setLanguage(config.language);
        }
        status = i18n.t("app.status.ready");
    } catch(e) {
        status = `${i18n.t("app.status.error")}: ${e}`;
    }
  }

  onMount(() => {
    loadState();
    
    // Listen for profile changes from hotkeys or auto-switch
    const unlisten = listen<{ index: number }>("profile-changed", (event) => {
        console.log("Profile changed via background task:", event.payload);
        activeIndex = event.payload.index;
    });
    
    return () => {
        unlisten.then(f => f());
    };
  });

  async function applyProfile(index: number) {
      status = i18n.t("app.status.applying");
      try {
          await invoke("apply_profile", { index });
          await loadState();
      } catch (e) {
          status = `${i18n.t("app.status.error")}: ${e}`;
      }
  }

  async function previewSettings(settings: DisplaySettings, displayIds: string[]) {
      try {
          await invoke("preview_settings", { settings, displayIds });
      } catch (e) {
          console.error("Preview failed:", e);
      }
  }

  async function saveProfile(updatedProfile: DisplayProfile) {
      if (editingIndex === null || !config) return;
      status = i18n.t("app.status.saving");
      try {
          // Update local config object
          const newProfiles = [...config.profiles];
          newProfiles[editingIndex] = updatedProfile;
          const newConfig = { ...config, profiles: newProfiles };
          
          // Save to backend - it will now automatically re-apply the last active profile by name
          await invoke("save_config", { updated: newConfig });
          
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
          const newProfiles = [...config.profiles];
          newProfiles.splice(editingIndex, 1);
          const newConfig = { ...config, profiles: newProfiles };
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

  async function handleReset() {
      if (!confirm(i18n.t("settings.confirm_reset"))) return;
      status = i18n.t("app.status.resetting");
      try {
          await invoke("reset_to_defaults");
          await loadState();
          status = i18n.t("app.status.ready");
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
  <div class="drag-handle" data-tauri-drag-region></div>
  <header class="header" data-tauri-drag-region>
    <div class="brand" data-tauri-drag-region>
      <img src="/icon.svg" alt="" class="logo" />
      <h1 data-tauri-drag-region>color-schemer</h1>
      {#if gpuInfo?.is_mock}
        <span class="mock-badge">{i18n.t('settings.status_mock')}</span>
      {/if}
    </div>
    <div class="header-right">
      <div class="status">{status}</div>
      <div class="window-controls">
        <button class="win-btn minimize" onclick={minimizeWindow} title="Minimize">
          <svg width="12" height="12" viewBox="0 0 12 12"><rect fill="currentColor" x="2" y="5.5" width="8" height="1"/></svg>
        </button>
        <button class="win-btn close" onclick={closeWindow} title="Close">
          <svg width="12" height="12" viewBox="0 0 12 12"><path fill="currentColor" d="M2.5,2.5 L9.5,9.5 M9.5,2.5 L2.5,9.5" stroke="currentColor" stroke-width="1.2"/></svg>
        </button>
      </div>
    </div>
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
                      profile={{ name: i18n.t('editor.new_profile'), description: "", settings: { brightness: 1, contrast: 1, gamma: 1, digital_vibrance: 0 }, target_displays: [], applications: [] }} 
                      {displays}
                      onSave={(p) => {
                          if (!config) return;
                          const newConfig = { 
                              ...config, 
                              profiles: [...config.profiles, p] 
                          };
                          saveSettings(newConfig).then(() => editingIndex = null);
                      }} 
                      onCancel={cancelEdit} 
                      onPreview={previewSettings} 
                  />
              {:else}
                  <ProfileEditor 
                      profile={config.profiles[editingIndex]} 
                      {displays}
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
          <SettingsTab {config} {gpuInfo} onSave={saveSettings} onReset={handleReset} />
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
    padding: 12px 18px 18px 18px;
    display: grid;
    gap: 14px;
    user-select: none;
    position: relative;
  }

  .drag-handle {
    position: absolute;
    top: 0;
    left: 0;
    right: 0;
    height: 60px;
    z-index: 0;
  }

  .header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 12px;
    padding-bottom: 5px;
    position: relative;
    z-index: 1;
    pointer-events: none;
  }
  
  .brand, .header-right {
    pointer-events: auto;
  }

  .brand {
    display: flex;
    align-items: center;
    gap: 12px;
  }
  .logo {
    width: 32px;
    height: 32px;
    filter: drop-shadow(0 0 5px var(--primary-glow));
  }
  h1 {
    margin: 0;
    font-size: 20px;
    color: var(--primary);
    text-transform: uppercase;
    letter-spacing: 1px;
    font-family: var(--font-mono);
  }
  .mock-badge {
    background: var(--error-dim);
    color: var(--text-main);
    font-size: 10px;
    font-family: var(--font-mono);
    padding: 2px 8px;
    border-radius: 4px;
    border: 1px solid var(--error);
    box-shadow: 0 0 5px var(--error-glow);
    text-transform: uppercase;
    margin-left: 8px;
  }
  .status {
    font-size: 11px;
    color: var(--primary);
    opacity: 0.8;
    font-family: var(--font-mono);
    text-transform: uppercase;
  }

  .header-right {
    display: flex;
    align-items: center;
    gap: 16px;
  }

  .window-controls {
    display: flex;
    gap: 4px;
    margin-right: -4px;
  }

  .win-btn {
    appearance: none;
    background: transparent;
    border: none;
    padding: 6px;
    border-radius: 6px;
    color: var(--text-muted);
    cursor: pointer;
    display: flex;
    align-items: center;
    justify-content: center;
    transition: all 0.2s ease;
  }

  .win-btn:hover {
    background: var(--bg-active);
    color: var(--primary);
    box-shadow: 0 0 8px var(--primary-glow);
  }

  .win-btn.close:hover {
    color: var(--error);
    box-shadow: 0 0 8px var(--error-glow);
  }

  .brand, h1 {
    cursor: default;
  }

  /* Basic tab styling */
  .tabs { display: flex; gap: 10px; margin-bottom: 20px; }
  .tabs button {
    appearance: none;
    background: var(--bg-layer);
    border: 1px solid var(--border);
    color: var(--text-muted);
    padding: 8px 16px;
    border-radius: 10px;
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
