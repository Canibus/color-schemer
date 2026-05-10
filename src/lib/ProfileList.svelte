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
    <div class="empty">{i18n.t('profiles.empty')}</div>
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
  <button class="add-btn" onclick={() => onEdit(-1)}>{i18n.t('profiles.add')}</button>
</div>

<style>
  .list { display: grid; gap: 12px; }
  
  .profile-row {
    display: flex;
    gap: 12px;
    background: var(--bg-layer);
    border: 1px solid var(--border);
    position: relative;
    overflow: hidden;
    transition: all 0.2s cubic-bezier(0.4, 0, 0.2, 1);
    padding: 12px;
    border-radius: 10px;
  }

  .profile-row:hover {
    border-color: var(--primary-dim);
    background: var(--bg-active);
  }

  .profile-row.selected {
    border-color: var(--primary);
    background: var(--bg-active);
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
    display: flex;
    flex-direction: column;
    gap: 4px;
  }

  .name { 
    font-weight: 600;
    color: var(--text-main);
    font-family: var(--font-mono);
    font-size: 14px;
    display: flex;
    align-items: center;
    text-transform: uppercase;
    letter-spacing: 0.5px;
  }

  .desc { 
    font-size: 11px;
    color: var(--text-muted);
    font-family: var(--font-sans);
    opacity: 0.8;
  }

  .edit-btn, .apply-action-btn { 
    padding: 6px 14px;
    background: var(--bg-surface);
    border: 1px solid var(--border);
    color: var(--text-main);
    cursor: pointer;
    border-radius: 6px;
    font-size: 11px;
    font-family: var(--font-mono);
    text-transform: uppercase;
    transition: all 0.2s ease;
    align-self: center;
    letter-spacing: 0.5px;
  }

  .apply-action-btn {
    border-color: var(--primary-dim);
    color: var(--primary);
    box-shadow: inset 0 -2px 0 var(--primary-dim);
  }

  .edit-btn:hover { 
    border-color: var(--primary);
    color: var(--primary);
    background: var(--bg-active);
  }

  .apply-action-btn:hover {
    border-color: var(--primary);
    background: var(--bg-active);
    box-shadow: var(--glow-shadow), inset 0 -1px 0 var(--primary-dim);
    transform: translateY(-1px);
  }
  
  .apply-action-btn:active {
    transform: translateY(1px);
    box-shadow: none;
  }

  .active-badge {
    margin-left: 8px;
    font-size: 9px;
    padding: 1px 6px;
    background: var(--primary);
    color: var(--bg-surface);
    border-radius: 4px;
    font-weight: 800;
    text-transform: uppercase;
    letter-spacing: 1px;
  }

  .actions { margin-top: 20px; display: flex; justify-content: center; }
  .add-btn { 
    padding: 12px;
    background: var(--bg-surface);
    border: 1px dashed var(--border);
    border-radius: 10px;
    color: var(--text-muted);
    cursor: pointer;
    width: 100%;
    font-family: var(--font-mono);
    text-transform: uppercase;
    font-size: 12px;
    letter-spacing: 1px;
    transition: all 0.2s ease;
  }
  .add-btn:hover { 
    border-color: var(--primary);
    color: var(--primary);
    background: var(--bg-active);
    box-shadow: var(--glow-shadow);
  }

  .empty {
    text-align: center;
    padding: 30px;
    color: var(--text-muted);
    border: 1px dashed var(--border);
    border-radius: 10px;
    font-family: var(--font-mono);
    text-transform: uppercase;
    font-size: 12px;
  }
</style>
