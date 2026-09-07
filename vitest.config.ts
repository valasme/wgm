import react from "@vitejs/plugin-react";
import { defineConfig } from "vitest/config";

// Deliberately not a merge of vite.config.ts: the router plugin's code-splitting
// transform and the React Compiler babel pass both cost time and neither is under
// test here. Tests exercise components, not the bundler.
export default defineConfig({
  plugins: [react()],
  test: {
    environment: "jsdom",
    globals: true,
    setupFiles: ["./src/test/setup.ts"],
    include: ["src/**/*.{test,spec}.{ts,tsx}"],
    css: false,
    restoreMocks: true,
    coverage: {
      provider: "v8",
      reporter: ["text", "lcov"],
      include: ["src/**/*.{ts,tsx}"],
      exclude: [
        "src/routeTree.gen.ts",
        "src/ipc/bindings.ts",
        "src/test/**",
        "src/**/*.{test,spec}.{ts,tsx}",
      ],
    },
  },
});
