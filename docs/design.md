# wgm — Design System

The visual direction is set by the mockup, not by this document. What follows makes that
mockup implementable: it fills in the palette, type scale, spacing and rules the plan
referred to but never defined.

Read `CONTEXT.md` for what things are called and `docs/adr/0002` for why schemes work the
way they do.

---

## 1. The design language, stated

The mockup commits to something specific and worth protecting: **a near-silent interface
with exactly one moment of personality.** Near-white surfaces, hairline divisions, no
shadows inside the window, no colour except where colour means something — and against all
that quiet, `wgm` set in a serif italic.

That restraint is the identity. Two rules follow from it, and most decisions below are
consequences of them:

1. **The serif appears in two places only** — the Title Bar wordmark and the onboarding
   welcome. It is not a display face for headings, empty states or the About page. Used
   three times it is a signature; used everywhere it is decoration.
2. **Colour carries meaning, never mood.** The Accent marks the focused, the selected and
   the primary action. Nothing is tinted to look nice.

---

## 2. Palette

A cool-neutral ramp — a trace of blue in the greys (hue 265 at very low chroma), matching
the mockup and deliberately not the warm off-white that every generated interface reaches
for. Five roles per scheme, and no colour outside this table except Accents.

| Role | Light | Dark | Darker |
| --- | --- | --- | --- |
| `--bg` | `oklch(99% 0 0)` | `oklch(20% 0.006 265)` | `oklch(0% 0 0)` |
| `--surface` | `oklch(96.5% 0.002 265)` | `oklch(24% 0.007 265)` | `oklch(13% 0.005 265)` |
| `--border` | `oklch(92% 0.003 265)` | `oklch(30% 0.008 265)` | `oklch(22% 0.006 265)` |
| `--text` | `oklch(20% 0.01 265)` | `oklch(95% 0.002 265)` | `oklch(94% 0.002 265)` |
| `--text-muted` | `oklch(50% 0.008 265)` | `oklch(68% 0.008 265)` | `oklch(64% 0.008 265)` |

**Darker is not Dark with the lightness turned down.** A design built on hairlines and
raised surfaces stops working on true black: a `#1A1A1A` divider on `#000` is invisible, and
a black panel on a black page has no edge. So in Darker, `--surface` sits *above* the
background rather than below it, and `--border` takes a proportionally larger step. Deriving
Darker arithmetically from Dark produces a scheme that looks broken; it has to be drawn.

**Accents** — Blue (default), Violet, Cyan, Emerald, Amber, Orange, Rose, Neutral. Each
supplies `--accent` and `--accent-foreground` per scheme. `scripts/check-contrast.ts` gates
**both** the eight accent pairs and the `--text` / `--text-muted` on `--bg` / `--surface`
pairs, in all three schemes, at WCAG AA. Checking only the accents would let the neutral
ramp — which is 95% of the pixels — drift unverified.

Under `forced-colors: active` the entire palette yields to the system. Accents do not
survive High Contrast, and should not try to.

---

## 3. Type

Two families, sharply distinct in role, never mixed within a component.

| Face | Role |
| --- | --- |
| Instrument Serif Italic | The wordmark. Two placements, listed in §1. |
| Geist Sans Variable | Everything else. |

Whole-pixel sizes, because Windows at 100% scaling renders fractional sizes softly:

| Token | Size / line-height | Use |
| --- | --- | --- |
| `--text-xs` | 11 / 16 | Keyboard hints, timestamps, file paths |
| `--text-sm` | 13 / 20 | **Base.** Body, every control, every label |
| `--text-md` | 15 / 22 | Section headings |
| `--text-lg` | 19 / 26 | Route `<h1>` |
| `--text-xl` | 28 / 34 | Onboarding step titles |
| wordmark | 16 | Instrument Serif Italic |

13px base rather than shadcn's 14px: this is a desktop tool sitting beside Windows' own
12–14px chrome, and a package manager is eventually a dense list.

Weights: 400 body, 500 for headings and active navigation. **No bold, no all-caps, no
tracked-out labels.** In an interface this quiet, a weight step from 400 to 500 is already a
strong signal.

**Density changes spacing, never type size.** Compact takes row height from 32 to 28 and
section gaps from 24 to 16. Shrinking 13px text to 12px to fit more in is a legibility cost
paid by the user for the developer's convenience.

---

## 4. Space, radius, elevation

Spacing scale: `4 · 8 · 12 · 16 · 24 · 32 · 48`. Nothing between.

Radii are graded by what the thing *is*, not applied uniformly:

| `--radius-sm` | 6px | Buttons, inputs, nav items, the search pill |
| `--radius-md` | 8px | Panels, dialogs, popovers |
| `--radius-lg` | 12px | The window itself |

**No shadows inside the window.** Depth comes from `--surface` and `--border`. The only
shadow in the product is the one Windows draws around the window, and floating layers
(dialogs, the command palette, tooltips) which need to read as *above* the page.

---

## 5. Layout rules

- **Settings rows cap at 640px** and sit 32px in from the Sidebar divider. At a 1200px
  window the Content Area is ~950px wide; a full-bleed row puts a switch most of a metre
  from the label it belongs to. This is the commonest layout mistake in desktop settings.
- **Route `<h1>` is 19px, left-aligned, 32px from the top.** No eyebrow label above it.
- **Content Area scrolls; the Title Bar and Sidebar never do.**
- **Below 900px window width the Sidebar forces to Rail.**

## 6. Selection, focus and state

The active navigation item is marked **two** ways: a filled `--surface` background and a 500
weight. Background tint alone fails "no information by colour alone", which is what the
weight is for; the Accent bar that used to sit on the leading edge is gone, because a third
cue on a control that already reads as selected is decoration.

Under `forced-colors: active` the fill *is* overridden, which would leave the weight alone —
so the active item takes a `Highlight` border there, and only there.

Focus ring: 2px `--accent` at 2px offset, on every focusable element, always visible on
keyboard focus. On a near-white page this ring is the loudest thing on screen — which is
correct, and is why the Accent is not used decoratively anywhere else.

Hover on nav and rows: `--surface` at 100ms. Nothing else hovers.

## 7. Motion

**One orchestrated moment: the transition out of onboarding into the app.** It happens once
per install and it is the only place motion is used to create an impression.

Everything else is functional and short: Sidebar collapse 150ms on width, Radix defaults for
dialogs and popovers, sonner's slide for toasts (bottom-right — top-right would collide with
the Window Controls). Colour Scheme changes are instant. Route changes are instant.

All of it is suppressed under `prefers-reduced-motion` or the in-app override.

---

## 8. Writing

Two rules that decide most of the copy in this app:

**The domain term and the interface term are allowed to differ.** `CONTEXT.md` calls the zip
a **Diagnostics Bundle** and that is what the code calls it. The button says
*Export diagnostics*, because the user's goal is filing a bug report, not learning our noun.

**An empty screen is an invitation to act, not a status report.** Every route in the
scaffold is empty, so the empty states *are* the product right now. "Search — coming soon"
is the wrong artefact to greet a contributor who has just cloned the repo. Each placeholder
renders the real intended empty state, with real copy:

| Route | Heading | Body | Action |
| --- | --- | --- | --- |
| `/search` | Find a package | Search winget for tools, runtimes and editors. | *(the search field)* |
| `/kits` | No kits yet | A kit is a set of packages you install together — a language runtime, its tooling and your editor. | Create a kit |
| `/kits/$kitId` | Kit not found | This kit isn't installed and isn't in the catalogue. | Back to kits |
| `/installed` | Nothing installed through wgm | Packages you install here will be listed, with their versions and sources. | Find a package |
| `/updates` | Everything's up to date | wgm checks installed packages against winget when you open this page. | Check again |
| `404` | That page doesn't exist | | Go to search |

Buttons name what happens and keep the same word afterwards: *Export settings* produces
*Settings exported*. Errors state what happened and what to do, and do not apologise:

> Settings couldn't be read, so wgm started with defaults.
> Your previous file is saved as `settings.corrupt-2026-09-07T14-22.json`.  **[Open folder]**

Labels that were jargon, corrected:

| Instead of | Say |
| --- | --- |
| Export logs / Export Diagnostics Bundle | Export diagnostics |
| Replay onboarding | Show setup again |
| Check for updates *(in About)* | Check for a new version |
| Settings → Data | Settings → Advanced |
| Submit / OK | The verb for what happens |

*Check for updates* is ambiguous the moment `/updates` exists and means Package Updates.
About checks for a **wgm Release**; the label has to say so on its own.
