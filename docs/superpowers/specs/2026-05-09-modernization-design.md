# Design Spec: Color Schemer Modernization (Gamer-Centric Prism)

**Date:** 2026-05-09
**Status:** Approved
**Aesthetic:** Gamer-Centric / High Performance
**Primary Motivation:** Align the application's visual identity with its technical purpose (NVIDIA display control) through a modern, aggressive, and memorable interface.

## 1. Aesthetic Thesis: "NVIDIA Neon"
The design adopts a "Hardware HUD" feel, utilizing deep obsidian backgrounds, sharp borders, and layered green glows. It should feel like a precision tool running on high-end hardware.

### 1.1 Color Palette
- **Obsidian (Surface):** `#0a0a0a` (Main background)
- **Deep Steel (Layer):** `#141414` (Cards, headers)
- **NVIDIA Green (Primary):** `#76b900` (Accents, active states, glows)
- **Dim Green (Secondary):** `#2d4a00` (Inactive states, secondary borders)
- **Pure White (Text):** `#ffffff` (Primary content)
- **Muted Steel (Text Muted):** `#888888` (Secondary content)

### 1.2 Typography
- **Headings/UI:** System default (Inter/Segoe UI) for readability.
- **Values/Technical Data:** `JetBrains Mono` or `Consolas`. All numeric values (brightness, contrast, etc.) must use monospaced fonts to avoid layout shifting and reinforce the "HUD" aesthetic.

## 2. Iconography: "The Prism"
The icon will be updated to a modern, abstract representation of light manipulation.

- **Concept:** A central geometric prism (triangle/diamond) hit by a white beam from the left, emitting three shades of green rays to the right.
- **Implementation:** 
    - Master SVG will be created and used for all Tauri icon generation.
    - SVG will utilize `<defs>` for a soft outer glow (`feGaussianBlur`).
    - The green rays will use a gradient from `#76b900` to transparent.

## 3. UI Components Refactor

### 3.1 Global Styles
- Introduce CSS variables for the palette.
- Implement a `glow` utility class using `box-shadow: 0 0 10px var(--primary-glow)`.
- Use `1px` solid borders for containers with slightly rounded corners (`8px`).

### 3.2 Profile List (`ProfileList.svelte`)
- Active profile row will have a left-side green "active bar" and a subtle green glow.
- Profile buttons will use a hover state that slightly increases border brightness.

### 3.3 Sliders & Controls (`ProfileEditor.svelte`)
- Range inputs will be custom-styled:
    - Track: Dark steel.
    - Fill: NVIDIA Green with a glow effect.
    - Thumb: High-contrast white or green circle.
- Values will be displayed prominently in mono-font next to the label.

## 4. Technical Implementation Notes
- **Transitions:** Use `0.2s ease-out` for all hover and active states.
- **Accessibility:** Ensure `#76b900` has sufficient contrast against `#0a0a0a` for essential information, or use white text with green shadows.
- **Tauri Integration:** Ensure `tauri.conf.json` is updated to point to the new icon assets once generated.

## 5. Success Criteria
- [ ] Interface feels "alive" through subtle glows and interactive feedback.
- [ ] The "Prism" icon is recognizable and distinct in the system tray.
- [ ] Typography remains legible while feeling technical.
- [ ] Performance remains high (no excessive filter effects).
