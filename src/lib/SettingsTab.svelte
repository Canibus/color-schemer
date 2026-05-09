<!-- src/lib/SettingsTab.svelte -->
<script lang="ts">
  import type { AppConfig } from "./types";
  export let config: AppConfig;
  export let onSave: (updated: AppConfig) => void;
</script>

<div>Settings Tab Placeholder</div>
