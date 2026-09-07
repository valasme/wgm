---
status: accepted
---

# Three Color Schemes, driven by a data attribute and OKLCH tokens

wgm ships three Color Schemes — Light, Dark and Darker (true black, for OLED) — selected via
`<html data-theme="light|dark|darker">` rather than the conventional `.dark` class, because
a boolean class cannot express three states. Every colour is an OKLCH CSS custom property
declared per scheme in `src/theme/tokens.css`, and each of the eight Accents supplies a
foreground pair per scheme.

`System` is a preference, not a scheme: it resolves to Light or Dark and **never** to
Darker, because the operating system has no signal for "extra dark" and nobody's OS is
asking for it. Darker is a deliberate opt-in.

## Consequences

- `color-scheme: dark` must be set on both dark variants, or Windows renders native
  scrollbars, focus rings and context menus in light colours inside a dark window.
- Accents cannot be a freeform colour picker. Eight fixed options are contrast-checked
  against all three scheme backgrounds by a CI script; a picker would let users choose a
  hue that fails WCAG AA and there would be no way to stop them.
- Scheme changes do not animate. A cross-fade forces a full repaint and always janks;
  native applications switch instantly.
