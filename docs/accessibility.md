# wgm — Accessibility Contract

What must be true for a release to ship. Anything here that can be checked by a machine is
checked in CI; the rest is a documented manual pass.

wgm's chrome is custom — a hand-built Title Bar, Window Controls and a resizable Sidebar
replace things Windows normally provides for free, along with the accessibility Windows
normally provides for free. Almost everything below exists because of that trade.

---

## 1. The window itself

**The OS window title is a real accessibility surface.** Alt+Tab, the taskbar and every
screen reader announce it on window focus. A single-page app does not update it on
navigation unless told to, so wgm calls `getCurrentWindow().setTitle()` on every route
change: `Settings · wgm`, `Search · wgm`. `<title>` is kept in sync for the same reason.
This is separate from, and additional to, the `aria-live` route announcement.

`Alt+Space` opens the real Windows system menu — undecorated windows lose it by default, and
it is how screen reader users have always reached window commands. `Alt+F4` and `Win+Arrow`
continue to work untouched.

## 2. Landmarks and headings

```
<header>            Title Bar — sidebar toggle, Wordmark, Window Controls
<nav aria-label>    Sidebar
<main id="main" tabindex="-1">   Content Area
```

One `<h1>` per route. **Settings is the exception**: its layout owns `<h1>Settings`, and each
of the four pages owns an `<h2>`. Focus on navigation therefore moves to *the heading that
changed* — the `<h2>` within Settings, the `<h1>` everywhere else. Sending focus to the `<h1>`
on a Settings sub-route change would announce "Settings" four times in a row and never name
the page the user actually opened.

## 3. Focus order

Skip link → sidebar toggle → Window Controls (one stop) → Sidebar nav → main.

**DOM order matches visual order.** An earlier draft of this plan put the Window Controls
last in the DOM and moved them with CSS, to keep them out of the way. That is a WCAG 2.4.3
violation for sighted keyboard users, who would watch focus jump from the bottom of the page
to the top-right corner. Two tab stops before the navigation is a smaller cost than a
disordered focus ring, and the skip link makes it one press to bypass both.

Window Controls are a single stop because they are a `role="toolbar"` with roving tabindex —
arrow keys move between minimise, maximise and close. Use Radix's Toolbar primitive rather
than hand-rolling the roving focus.

## 4. Focus indication

**Focus rings use `outline`, never `box-shadow`.** Under Windows High Contrast
(`forced-colors: active`) box-shadows are stripped entirely and an outline is re-coloured to
the system highlight — so a `box-shadow` ring vanishes for exactly the users who most need
it. 2px outline, 2px offset, `--accent`.

## 5. The Sidebar resizer

It is a splitter, and it implements the ARIA window-splitter pattern in full:

```
role="separator"  aria-orientation="vertical"  tabindex="0"
aria-label="Sidebar width"
aria-valuenow / aria-valuemin / aria-valuemax
aria-valuetext="220 pixels"
```

Arrow keys move it in 16px steps; Home and End jump to the bounds.

**Hit area is 10px, not 6px.** WCAG 2.5.8 asks for 24×24 and a splitter cannot be that wide
without changing the layout it exists to control — the Essential exception applies — but 6px
is a hard target to hit with a mouse for anyone with a motor impairment, and 10px costs
nothing visually since the rendered line stays 1px.

## 6. Colour and contrast

Verified in CI by `scripts/check-contrast.ts`: eight Accent pairs plus the neutral text ramp,
across all three Color Schemes, at WCAG AA.

**Nothing is signalled by colour alone.** The active navigation item carries two cues — a
`--surface` fill and 500 weight — and `aria-current="page"`. The weight is the one that does
the work: in an interface this quiet, a tint this subtle is missable with full colour vision,
let alone without it.

Under `forced-colors: active` the palette yields entirely to the system. Two consequences to
build for: the active-nav indicator must gain a `border`/`outline` there, because backgrounds
are overridden and borders are re-coloured — this is the one mode where the fill cannot be
relied on; and the Window Control glyphs must be drawn with `currentColor`, not a hardcoded
fill, or they disappear.

## 7. Targets and zoom

Minimum interactive target 24×24, **including in Compact density** — Compact reduces row
height to 28px and section gaps to 16px, and must not be allowed to shrink icon buttons
below the floor. The Window Control glyphs are 10×10 artwork inside buttons that are at
least 24×32.

`Ctrl` `+` / `-` / `0` zoom is enabled explicitly (off by default in Tauri) as the cheapest
low-vision accommodation available. No fixed heights on text containers, so a 1.5× line-height
override does not clip.

## 8. Screen reader behaviour under instant apply

Settings apply immediately, with rollback on failure. That creates an announcement problem
unique to this pattern: a screen reader says "on", Rust rejects the write, the control
silently returns to "off", and the user is never told.

**A rollback is always announced.** The failure is surfaced three ways, not one:

1. the control returns to its previous state,
2. an **inline** error message appears on that row, programmatically associated with the
   control via `aria-describedby` — WCAG 3.3.1 wants the error next to the thing that failed,
   and a toast that has already faded fails anyone who looked away,
3. a toast, as the ambient signal.

## 9. Onboarding is not a modal

**It does not trap focus.** An earlier draft said it did. Onboarding is a full route with
nothing behind it, so a trap buys nothing — and it would lock the user away from the Window
Controls, leaving them unable to minimise or close the app during setup. Focus starts on the
step heading; Tab reaches the Window Controls as usual.

Progress dots use `aria-current="step"`. Every step is completable by keyboard, and Skip is
a real focusable control, not a corner affordance.

## 10. Discoverability without a keyboard

The `Ctrl+/` shortcut sheet must also be reachable by pointer — a keyboard-only route to the
list of keyboard shortcuts helps nobody who does not already know it. It appears in About.

Rail navigation items carry tooltips *and* `aria-label`s. Tooltips are not reliably announced
and are never the accessible name.

## 11. What CI checks, and what it cannot

| Checked by | Covers |
| --- | --- |
| `axe-core` (jsdom), per route | Roles, accessible names, labels, ARIA validity, landmarks, heading order |
| `scripts/check-contrast.ts` | All colour pairs, three schemes |
| Manual pass, per release | 200% display scaling · Windows High Contrast · full no-mouse pass · one screen reader pass with NVDA or Narrator |

**axe in jsdom cannot check colour contrast.** jsdom computes no layout, so the rule silently
no-ops and returns green. Reading a passing axe run as contrast coverage is the specific
mistake this table exists to prevent.
