# Regenerating the wordmark subset

The Wordmark — `wgm` set in Instrument Serif Italic — is the only serif in the
product, and it is three glyphs. `public/fonts/instrument-serif-italic-wgm.woff2`
contains exactly `w`, `g` and `m`.

## Why a subset at all

wgm self-hosts every font: the zero-network policy
([ADR-0003](../docs/adr/0003-zero-network-policy.md)) rules out a font CDN, and the
Content-Security-Policy sets `font-src 'self'`, so a CDN link would silently fail
anyway. Self-hosting the *full* Instrument Serif Italic would ship roughly 30 KB of
glyphs that are never drawn.

## How

```bash
pnpm subset-wordmark
```

That runs [`scripts/subset-wordmark.ts`](subset-wordmark.ts), which reads the upstream
woff2 from the `@fontsource/instrument-serif` dev dependency, subsets it with
`subset-font` (harfbuzz), and writes the result into `public/fonts/`. The output is
checked in, so a normal build and a normal clone never need to run it.

Regenerate when:

- `@fontsource/instrument-serif` is bumped to a version with real glyph changes, or
- the Wordmark ever needs a character it does not currently have. It will not: the
  product is called `wgm` and the name is fixed in `CONTEXT.md`.

## The licence obligation

Instrument Serif is licensed under the **SIL Open Font License 1.1**, and the OFL is
explicit that a subset is a Modified Version. Three consequences, all already handled:

1. **The licence text ships with the font.** `licenses/OFL-InstrumentSerif.txt` is in
   the repo and is folded into `THIRD-PARTY-LICENSES.md` by the `licenses.yml`
   workflow. Do not remove it because the file "is only three glyphs" — the OFL makes
   no size exemption.
2. **The Reserved Font Name is not reused.** The output file is named
   `instrument-serif-italic-wgm.woff2` and the CSS `font-family` is
   `"Instrument Serif Subset"`, deliberately not `"Instrument Serif"`. The OFL forbids
   distributing a modified font under the original Reserved Font Name.
3. **It stays under the OFL and cannot be sold on its own.** Neither is a constraint
   wgm was ever going to hit.

The same applies to Geist, which ships whole rather than subsetted:
`licenses/OFL-Geist.txt`.

## Checking the result

```bash
pnpm subset-wordmark
```

prints the before and after sizes. Then look at the Title Bar in `pnpm tauri dev`: the
Wordmark renders in the serif, and the fallback stack (`Georgia, serif`) is what you
see if the subset failed to load. If the Wordmark is upright rather than italic, the
`font-style: italic` descriptor on the `@font-face` is missing — the subset contains
only the italic face, so nothing synthesises it.
