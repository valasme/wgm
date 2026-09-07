import { createFileRoute, redirect } from "@tanstack/react-router";

/**
 * `/` is not a page.
 *
 * Search is where a package manager starts, so the root redirects there rather than
 * inventing a dashboard with nothing on it.
 */
export const Route = createFileRoute("/")({
  beforeLoad: () => {
    throw redirect({ to: "/search" });
  },
});
