# Design Doc: Settings Tab Refactor

## Goal
Update the `SettingsTab.svelte` component to match the HUD aesthetic, featuring "mechanical keycap" styled hotkey buttons and a high-tech terminal look.

## Requirements
- Update `SettingsTab.svelte` with new styles.
- Use CSS variables from `theme.css`.
- Implement a "keycap" look for hotkey buttons using `box-shadow: inset 0 -2px 0 var(--primary-dim)`.
- Use a terminal-like appearance for fieldsets and headers.

## Approach
- Replace the existing `<style>` block in `SettingsTab.svelte` with the provided CSS.
- Ensure the HTML structure matches the new CSS selectors (e.g., `.settings`, `fieldset`, `legend`, `.hotkey-item button`, `.save-btn`).
- Verify that the `recording` state visual feedback is consistent with the "glow" effect.

## Proposed Changes

### `src/lib/SettingsTab.svelte`
- Replace existing styles with the new aesthetic.
- Update the layout slightly if necessary to fit the new padding and margins.

## Verification Plan
- Manual check: Verify that the buttons look like keycaps (inset shadow).
- Manual check: Verify that headers are uppercase and primary color.
- Manual check: Verify that fieldsets have the correct background and border.
