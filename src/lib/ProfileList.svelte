<!-- src/lib/ProfileList.svelte -->
<script lang="ts">
  import type { DisplayProfile } from "./types";
  export let profiles: DisplayProfile[] = [];
  export let activeIndex: number | null = null;
  export let onApply: (index: number) => void;
  export let onEdit: (index: number) => void;
</script>

<section class="list">
  {#if profiles.length === 0}
    <div class="empty">No profiles found.</div>
  {:else}
    {#each profiles as p, i}
      <div class="profile-row" class:selected={activeIndex === i}>
          <button class="apply-btn" on:click={() => onApply(i)} title={p.description}>
            <div class="name">{p.name}</div>
            <div class="desc">{p.description}</div>
          </button>
          <button class="edit-btn" on:click={() => onEdit(i)}>Edit</button>
      </div>
    {/each}
  {/if}
</section>

<div class="actions">
  <button class="add-btn" on:click={() => onEdit(-1)}>+ Add New Profile</button>
</div>

<style>
  .list { display: grid; gap: 10px; }
  .profile-row { display: flex; gap: 10px; border: 1px solid #2a3440; border-radius: 10px; padding: 5px; background: #121821; }
  .profile-row.selected { border-color: #6fb1ff; }
  .apply-btn { flex: 1; text-align: left; background: transparent; border: none; color: inherit; cursor: pointer; padding: 5px; }
  .edit-btn { padding: 5px 10px; background: #2a3440; border: none; border-radius: 5px; color: inherit; cursor: pointer; }
  .edit-btn:hover { background: #3a4b5d; }
  .name { font-weight: 650; }
  .desc { font-size: 12px; opacity: 0.85; }

  .actions { margin-top: 15px; display: flex; justify-content: center; }
  .add-btn { padding: 8px 15px; background: #1f2937; border: 1px dashed #4b5563; border-radius: 5px; color: inherit; cursor: pointer; width: 100%; }
  .add-btn:hover { background: #374151; border-color: #9ca3af; }
</style>
