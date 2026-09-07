import { createFileRoute } from "@tanstack/react-router";
import { SearchIcon } from "lucide-react";

import { PagePlaceholder } from "@/components/states/page-placeholder";

export const Route = createFileRoute("/_app/search")({
  component: SearchPage,
});

/**
 * The real empty state, not a "coming soon" screen: the search field is what belongs
 * here, and wgm does not invoke winget yet, so the field is not rendered. A control
 * that does nothing is a dead control.
 */
function SearchPage() {
  return (
    <PagePlaceholder
      title="route.search.title"
      heading="route.search.heading"
      body="route.search.body"
      icon={<SearchIcon className="size-8" aria-hidden="true" />}
    />
  );
}
