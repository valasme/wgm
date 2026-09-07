import { createFileRoute } from "@tanstack/react-router";
import { RefreshCwIcon } from "lucide-react";

import { PagePlaceholder } from "@/components/states/page-placeholder";

export const Route = createFileRoute("/_app/updates")({
  component: UpdatesPage,
});

/**
 * Package Updates, not a wgm Release — the two are different things and `/updates`
 * owns the first. Settings → About checks for the second.
 *
 * No "Check again" button: nothing to check against until wgm drives winget, and a
 * button that does nothing is a dead control.
 */
function UpdatesPage() {
  return (
    <PagePlaceholder
      title="route.updates.title"
      heading="route.updates.heading"
      body="route.updates.body"
      icon={<RefreshCwIcon className="size-8" aria-hidden="true" />}
    />
  );
}
