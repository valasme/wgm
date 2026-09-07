import { en } from "./en";

/**
 * The message catalog and its lookup function.
 *
 * **Rust never sends English to the UI.** Errors cross the boundary carrying a machine
 * code and structured context; this file owns every user-facing word, and `t()` takes
 * typed interpolation parameters from day one because those Rust errors arrive with
 * context like `{ path }` already attached.
 *
 * There is exactly one catalog. A language selector is deliberately **not rendered**
 * until a second one exists — see the no-dead-controls rule in docs/PLAN.md §Phase 5.
 */

export type Catalog = typeof en;
export type MessageKey = keyof Catalog;

/**
 * Extracts `{name}` placeholders from a message at the type level, so
 * `t("errors.DOCUMENT_CORRUPT.body")` will not compile without a `backup`.
 */
type Placeholders<Message extends string> =
  Message extends `${string}{${infer Name}}${infer Rest}`
    ? { [Key in Name]: string | number } & Placeholders<Rest>
    : // `unknown` rather than `{}` for the base case: it is the identity for `&`, so
      // it disappears from an intersection, and `keyof unknown` is `never`, which is
      // what makes a message with no placeholders take no argument at all.
      unknown;

type ParamsFor<Key extends MessageKey> = Placeholders<Catalog[Key]>;

type ArgsFor<Key extends MessageKey> = keyof ParamsFor<Key> extends never
  ? []
  : [params: ParamsFor<Key>];

/**
 * Look up a message and interpolate its parameters.
 *
 * A key that is not in the catalog is a compile error. A missing parameter is a
 * compile error. Neither can reach a user.
 */
export function t<Key extends MessageKey>(key: Key, ...args: ArgsFor<Key>): string {
  const template: string = en[key];
  const params = args[0] as Record<string, string | number> | undefined;

  if (params === undefined) {
    return template;
  }

  return template.replace(/\{(\w+)\}/g, (whole, name: string) => {
    const value = params[name];
    // Leaving the placeholder visible beats rendering "undefined": it says plainly
    // that something is missing rather than asserting a value that is not there.
    return value === undefined ? whole : String(value);
  });
}

/**
 * Look up a message whose key is only known at runtime — a Rust `ErrorCode`, say.
 *
 * Returns `undefined` rather than throwing when the key is unknown, so a code from a
 * newer build degrades to the generic message instead of taking the screen down.
 */
export function tryTranslate(
  key: string,
  params?: Record<string, string | number>,
): string | undefined {
  if (!(key in en)) {
    return undefined;
  }

  const template = en[key as MessageKey] as string;

  if (params === undefined) {
    return template;
  }

  return template.replace(/\{(\w+)\}/g, (whole, name: string) => {
    const value = params[name];
    return value === undefined ? whole : String(value);
  });
}
