<!-- src/lib/ProfileEditor.svelte -->
<script lang="ts">
  import type { DisplayProfile } from "./types";
  export let profile: DisplayProfile;
  export let onSave: (updated: DisplayProfile) => void;
  export let onCancel: () => void;
  export let onPreview: (settings: any) => void;
</script>

<div>Profile Editor Placeholder</div>
