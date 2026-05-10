<!-- src/lib/ProfileList.svelte -->
<script lang="ts">
  import type { DisplayProfile } from "./types";
  import { i18n } from "./i18n.svelte";
  
  let { 
    profiles = [], 
    activeIndex = null, 
    onApply, 
    onEdit 
  } = $props<{
    profiles: DisplayProfile[];
    activeIndex: number | null;
    onApply: (index: number) => void;
    onEdit: (index: number) => void;
  }>();
</script>

<section class="list">
  {#if profiles.length === 0}
    <div class="empty">No profiles found.</div>
  {:else}
    {#each profiles as p, i}
      <div class="profile-row" class:selected={activeIndex === i}>
          <button class="apply-btn" onclick={() => onApply(i)} title={p.description}>
            <div class="name">
              {p.name}
              {#if activeIndex === i}
                <span class="active-badge">{i18n.t('profiles.active')}</span>
              {/if}
            </div>
            <div class="desc">{p.description}</div>
          </button>
          {#if activeIndex !== i}
            <button class="apply-action-btn" onclick={() => onApply(i)}>{i18n.t('profiles.apply')}</button>
          {/if}
          <button class="edit-btn" onclick={() => onEdit(i)}>{i18n.t('profiles.edit')}</button>
      </div>
    {/each}
  {/if}
</section>

<div class="actions">
  <button class="add-btn" onclick={() => onEdit(-1)}>+ Add New Profile</button>
</div>

<style>
  .list { display: grid; gap: 10px; }
  
  .profile-row {
    display: flex;
    gap: 10px;
    background: var(--bg-layer);
    border: 1px solid var(--border);
    position: relative;
    overflow: hidden;
    transition: all 0.2s ease-out;
    padding: 10px;
    border-radius: 8px;
  }

  .profile-row:hover {
    border-color: var(--text-muted);
  }

  .profile-row.selected {
    border-color: var(--primary);
    box-shadow: var(--glow-shadow);
  }

  .profile-row.selected::before {
    content: '';
    position: absolute;
    left: 0; top: 0; bottom: 0;
    width: 4px;
    background: var(--primary);
    box-shadow: 2px 0 10px var(--primary-glow);
  }

  .apply-btn { 
    flex: 1; 
    text-align: left; 
    background: transparent; 
    border: none; 
    color: inherit; 
    cursor: pointer; 
    padding: 0; 
  }

  .name { 
    font-weight: 650;
    color: var(--text-main);
    font-family: var(--font-mono);
  }

  .desc { 
    font-size: 12px;
    color: var(--text-muted);
  }

  .edit-btn { 
    padding: 5px 12px;
    background: var(--bg-surface);
    border: 1px solid var(--border);
    color: var(--text-main);
    cursor: pointer;
    border-radius: 8px;
    font-size: 12px;
    transition: all 0.2s ease;
    align-self: center;
  }

  .edit-btn:hover { 
    border-color: var(--primary);
    color: var(--primary);
  }

  .apply-action-btn {
    padding: 5px 12px;
    background: var(--bg-surface);
    border: 1px solid var(--border);
    color: var(--text-main);
    cursor: pointer;
    border-radius: 8px;
    font-size: 12px;
    transition: all 0.2s ease;
    align-self: center;
  }

  .apply-action-btn:hover {
    border-color: var(--primary);
    color: var(--primary);
  }

  .active-badge {
    margin-left: 8px;
    font-size: 10px;
    padding: 2px 6px;
    background: var(--primary);
    color: var(--bg-base);
    border-radius: 4px;
    font-weight: bold;
    text-transform: uppercase;
  }

  .actions { margin-top: 15px; display: flex; justify-content: center; }
  .add-btn { 
    padding: 10px 15px;
    background: var(--bg-surface);
    border: 1px dashed var(--border);
    border-radius: 8px;
    color: var(--text-muted);
    cursor: pointer;
    width: 100%;
    font-family: var(--font-mono);
    transition: all 0.2s ease;
  }
  .add-btn:hover { 
    border-color: var(--primary);
    color: var(--primary);
    background: var(--bg-layer);
  }

  .empty {
    text-align: center;
    padding: 20px;
    color: var(--text-muted);
    border: 1px dashed var(--border);
    border-radius: 8px;
  }
</style>
