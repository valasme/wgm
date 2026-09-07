// vitest-axe ships an augmentation for Vitest 1's `Vi` namespace, which Vitest 5 no
// longer reads. Declaring the matcher against the current interface is the whole fix.
import "vitest";

declare module "vitest" {
  interface Matchers<R = void> {
    toHaveNoViolations(): R;
  }
}
