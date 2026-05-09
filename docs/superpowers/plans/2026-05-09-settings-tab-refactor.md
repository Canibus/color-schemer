# Settings Tab Refactor Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Refactor the Settings Tab to match the HUD aesthetic with mechanical keycap styled hotkey buttons and high-tech terminal visuals.

**Architecture:** Update Svelte component styles and structure to use CSS variables from `theme.css`.

**Tech Stack:** Svelte 5, TypeScript, CSS.

---

### Task 1: Refactor SettingsTab.svelte Styles

**Files:**
- Modify: `src/lib/SettingsTab.svelte`

- [ ] **Step 1: Update styles with mechanical keycap and terminal aesthetic**

```svelte
<style>
  .settings { 
    color: var(--text-main); 
    display: flex; 
    flex-direction: column; 
    gap: 15px; 
  }
  h3 { 
    color: var(--primary); 
    text-transform: uppercase; 
    font-size: 16px; 
    margin-top: 0; 
  }
  fieldset {
    background: var(--bg-layer);
    border: 1px solid var(--border);
    border-radius: 8px;
    padding: 18px;
    margin-bottom: 15px;
    display: flex;
    flex-direction: column;
    gap: 12px;
  }
  legend {
    color: var(--primary);
    font-family: var(--font-mono);
    font-size: 12px;
    text-transform: uppercase;
    padding: 0 8px;
  }
  .hotkey-item { 
    display: flex; 
    align-items: center; 
    justify-content: space-between; 
    gap: 10px; 
  }
  .hotkey-item span { 
    font-size: 14px; 
  }
  .hotkey-item button {
    background: var(--bg-surface);
    border: 1px solid var(--primary-dim);
    color: var(--primary);
    font-family: var(--font-mono);
    padding: 8px 16px;
    border-radius: 6px;
    cursor: pointer;
    min-width: 140px;
    text-align: center;
    font-size: 13px;
    transition: all 0.2s;
    box-shadow: inset 0 -2px 0 var(--primary-dim); /* "Keycap" look */
  }
  .hotkey-item button:hover {
    border-color: var(--primary);
    background: var(--bg-active);
  }
  .hotkey-item button.recording {
    border-color: var(--primary);
    box-shadow: var(--glow-shadow), inset 0 0 5px var(--primary-glow);
    color: #fff;
  }
  .checkbox-label { 
    display: flex; 
    align-items: center; 
    gap: 10px; 
    font-size: 14px; 
    cursor: pointer; 
  }
  .actions { 
    display: flex; 
    justify-content: flex-end; 
    margin-top: 10px; 
  }
  .save-btn {
    background: var(--primary);
    color: var(--bg-surface);
    border: none;
    padding: 10px 24px;
    border-radius: 8px;
    font-weight: bold;
    cursor: pointer;
    transition: all 0.2s;
  }
  .save-btn:hover {
    filter: brightness(1.1);
    box-shadow: var(--glow-shadow);
  }
</style>
```

- [ ] **Step 2: Commit settings refactor**

Run:
```bash
git add src/lib/SettingsTab.svelte
git commit -m "style: refactor SettingsTab with terminal-style hotkeys"
```
