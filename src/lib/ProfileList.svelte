<!-- src/lib/ProfileList.svelte -->
<script lang="ts">
  import type { DisplayProfile } from "./types";
  export let profiles: DisplayProfile[] = [];
  export let activeIndex: number | null = null;
  export let onApply: (index: number) => void;
  export let onEdit: (index: number) => void;
</script>

<div>Profile List Placeholder</div>
