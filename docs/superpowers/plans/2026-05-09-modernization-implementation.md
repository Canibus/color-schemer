# Gamer-Centric Prism Modernization Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Modernize the Color Schemer UI and brand identity using an NVIDIA-inspired "Gamer-Centric" aesthetic and a new "Prism" icon.

**Architecture:** Centralized design system using CSS variables (`src/theme.css`), custom styled Svelte components for a "HUD" feel, and a new SVG-based master icon.

**Tech Stack:** Svelte 5, CSS Variables, SVG, Tauri.

---

### Task 1: Create the "Prism" Master Icon

**Files:**
- Modify: `src-tauri/icons/icon.svg`

- [ ] **Step 1: Write the new SVG content**
Replace the current placeholder with the Prism design.

```svg
<svg width="512" height="512" viewBox="0 0 512 512" fill="none" xmlns="http://www.w3.org/2000/svg">
  <defs>
    <filter id="glow" x="-20%" y="-20%" width="140%" height="140%">
      <feGaussianBlur stdDeviation="15" result="blur" />
      <feComposite in="SourceGraphic" in2="blur" operator="over" />
    </filter>
    <linearGradient id="ray-grad" x1="0%" y1="50%" x2="100%" y2="50%">
      <stop offset="0%" stop-color="#76b900" stop-opacity="0.8" />
      <stop offset="100%" stop-color="#76b900" stop-opacity="0" />
    </linearGradient>
  </defs>
  <!-- Background Rays -->
  <path d="M256 256 L500 180" stroke="url(#ray-grad)" stroke-width="20" filter="url(#glow)" />
  <path d="M256 256 L500 256" stroke="url(#ray-grad)" stroke-width="25" filter="url(#glow)" />
  <path d="M256 256 L500 332" stroke="url(#ray-grad)" stroke-width="20" filter="url(#glow)" />
  <!-- Incoming Beam -->
  <path d="M20 256 L256 256" stroke="white" stroke-width="8" stroke-dasharray="10 5" opacity="0.6" />
  <!-- The Prism -->
  <path d="M256 120 L360 320 L152 320 Z" fill="#141414" stroke="#76b900" stroke-width="12" filter="url(#glow)" />
</svg>
```

- [ ] **Step 2: Commit icon update**
```bash
git add src-tauri/icons/icon.svg
git commit -m "design: implement new Prism master icon"
```

### Task 2: Define "NVIDIA Neon" Design System

**Files:**
- Create: `src/theme.css`
- Modify: `src/App.svelte`

- [ ] **Step 1: Create theme.css with variables**
```css
:root {
  --bg-surface: #0a0a0a;
  --bg-layer: #141414;
  --bg-active: #1a1a1a;
  --primary: #76b900;
  --primary-dim: #2d4a00;
  --primary-glow: rgba(118, 185, 0, 0.4);
  --text-main: #ffffff;
  --text-muted: #888888;
  --border: #2a3440;
  --font-mono: 'JetBrains Mono', 'Consolas', monospace;
  --glow-shadow: 0 0 15px var(--primary-glow);
}

:global(body) {
  background-color: var(--bg-surface) !important;
  color: var(--text-main) !important;
}
```

- [ ] **Step 2: Import theme in App.svelte**
Add `import "./theme.css";` to the script tag and remove redundant `:global(body)` styles.

- [ ] **Step 3: Commit design system**
```bash
git add src/theme.css src/App.svelte
git commit -m "style: establish NVIDIA Neon design system variables"
```

### Task 3: Refactor App Shell & Tabs

**Files:**
- Modify: `src/App.svelte`

- [ ] **Step 1: Update Header and Tab styling**
Apply the new variables and "glow" active states to the tab buttons.

```svelte
<!-- In <style> -->
.tabs button {
  background: var(--bg-layer);
  border: 1px solid var(--border);
  color: var(--text-muted);
  transition: all 0.2s ease-out;
}
.tabs button.active {
  color: var(--primary);
  border-color: var(--primary);
  box-shadow: var(--glow-shadow);
  background: var(--bg-active);
}
h1 { color: var(--primary); text-transform: uppercase; letter-spacing: 1px; }
```

- [ ] **Step 2: Commit shell update**
```bash
git add src/App.svelte
git commit -m "style: modernize app shell and navigation"
```

### Task 4: Refactor Profile List

**Files:**
- Modify: `src/lib/ProfileList.svelte`

- [ ] **Step 1: Update row styling for "HUD" feel**
Add the active indicator bar and glow effect.

```svelte
<!-- In <style> -->
.profile-row {
  background: var(--bg-layer);
  border: 1px solid var(--border);
  position: relative;
  overflow: hidden;
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
  box-shadow: 2px 0 10px var(--primary);
}
.name { color: var(--text-main); font-family: var(--font-mono); }
```

- [ ] **Step 2: Commit list refactor**
```bash
git add src/lib/ProfileList.svelte
git commit -m "style: refactor ProfileList with HUD-style rows"
```

### Task 5: Refactor Profile Editor (Controls)

**Files:**
- Modify: `src/lib/ProfileEditor.svelte`

- [ ] **Step 1: Implement custom Range Sliders**
```svelte
<!-- In <style> -->
input[type="range"] {
  accent-color: var(--primary);
}
.value-display {
  font-family: var(--font-mono);
  color: var(--primary);
  font-weight: bold;
}
fieldset {
  background: var(--bg-layer);
  border: 1px solid var(--border);
}
```

- [ ] **Step 2: Commit editor refactor**
```bash
git add src/lib/ProfileEditor.svelte
git commit -m "style: refactor ProfileEditor with mono-fonts and custom sliders"
```

### Task 6: Refactor Settings Tab

**Files:**
- Modify: `src/lib/SettingsTab.svelte`

- [ ] **Step 1: Update Hotkey buttons**
Make them look like mechanical keycaps or high-tech terminals.

```svelte
<!-- In <style> -->
.hotkey-item button {
  background: var(--bg-surface);
  border: 1px solid var(--primary-dim);
  color: var(--primary);
  font-family: var(--font-mono);
}
.hotkey-item button.recording {
  border-color: var(--primary);
  box-shadow: var(--glow-shadow);
  color: #fff;
}
```

- [ ] **Step 2: Commit settings refactor**
```bash
git add src/lib/SettingsTab.svelte
git commit -m "style: refactor SettingsTab with terminal-style hotkeys"
```

### Task 7: Verification & Visual Polish

- [ ] **Step 1: Run dev environment**
`bun run tauri dev`
Check all screens for consistency, glow intensity, and font legibility.

- [ ] **Step 2: Final Commit**
```bash
git commit --allow-empty -m "chore: finalize modernization visual polish"
```
