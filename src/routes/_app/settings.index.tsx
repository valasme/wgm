import { createFileRoute, redirect } from "@tanstack/react-router";

/**
 * `/settings` is a layout, not a page. Appearance is the first tab and the one people
 * open Settings for, so it is where the bare path lands.
 */
export const Route = createFileRoute("/_app/settings/")({
  beforeLoad: () => {
    throw redirect({ to: "/settings/appearance" });
  },
});
