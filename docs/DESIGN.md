# Design Specification: Color Schemer

**Date:** 2026-05-09
**Status:** Approved / Implemented
**Aesthetic:** "NVIDIA Neon" / Gamer-Centric HUD
**Primary Motivation:** Align the visual identity with technical display control via a modern, aggressive, and memorable interface.

---

## 1. Aesthetic Thesis: "NVIDIA Neon"
The design adopts a "Hardware HUD" feel, utilizing deep obsidian backgrounds, sharp borders, and layered green glows. It should feel like a precision tool running on high-end hardware.

### 1.1 Color Palette (CSS Variables)
All styles must reference the variables defined in `src/theme.css`:
- **`--bg-surface`:** `#0a0a0a` (Main background)
- **`--bg-layer`:** `#141414` (Cards, containers)
- **`--primary`:** `#76b900` (NVIDIA Green - accents, active states, glows)
- **`--primary-dim`:** `#2d4a00` (Inactive states, keycap shadows)
- **`--text-main`:** `#ffffff` (Primary content)
- **`--text-muted`:** `#888888` (Secondary content)

### 1.2 Typography
- **Headings/UI:** System default (Inter/Segoe UI) for readability.
- **Values/Technical Data:** `JetBrains Mono` or `Consolas`. All numeric values and hotkeys must use monospaced fonts to reinforce the technical aesthetic.

---

## 2. Iconography: "The Prism"
The brand icon represents light manipulation through a geometric prism.

- **Concept:** A central prism hit by white light, emitting three shades of green rays.
- **Implementation:** 
    - Master SVG: `src-tauri/icons/icon.svg`.
    - Assets: Generated PNGs in `src-tauri/icons/`.
    - Frontend serving: `public/icon.svg` (for Vite/Header).

---

## 3. UI Architecture (HUD Components)

### 3.1 Global HUD Style
Instead of standard OS-level controls, the app uses custom "Hardware HUD" containers:
- **`.field-group`:** Replaces standard `fieldset`. Features a `.group-label` header with a contrasting background and a `.group-content` body.
- **Glow Utilities:** `var(--glow-shadow)` is used to highlight active profiles and recording states.

### 3.2 Component Details
- **Profile List:** Active profiles feature a left-side 4px "active bar" and a subtle glow.
- **Profile Editor:** Custom range sliders using `accent-color: var(--primary)`. Values are displayed in mono-font labels.
- **Settings Tab:** 
    - **Hotkeys:** Styled as "mechanical keycaps" using `inset 0 -2px 0 var(--primary-dim)`.
    - **Recording State:** Pulse-like glow effect when a hotkey is being captured.

---

## 4. Technical Implementation Notes
- **Svelte 5:** Uses runes (`$state`, `$props`, `$effect`) for reactive state.
- **Transitions:** Standardized `0.2s ease-out` for all interactive states.
- **Syncing:** Editor components must use `$effect` to synchronize local edited state with incoming props to avoid stale data.
