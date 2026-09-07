/**
 * `subset-font` ships no types and has no `@types/` package. It is used in exactly one
 * place — `scripts/subset-wordmark.ts` — and only ever with these two arguments, so a
 * hand-written declaration is smaller and more honest than an `any`.
 */
declare module "subset-font" {
  export default function subsetFont(
    font: Buffer,
    text: string,
    options?: { targetFormat?: "woff2" | "woff" | "truetype" | "sfnt" },
  ): Promise<Buffer>;
}
