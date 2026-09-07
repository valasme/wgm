import { createFileRoute } from "@tanstack/react-router";
import { BoxIcon } from "lucide-react";

import { PagePlaceholder } from "@/components/states/page-placeholder";

export const Route = createFileRoute("/_app/kits/")({
  component: KitsPage,
});

function KitsPage() {
  return (
    <PagePlaceholder
      title="route.kits.title"
      heading="route.kits.heading"
      body="route.kits.body"
      icon={<BoxIcon className="size-8" aria-hidden="true" />}
    />
  );
}
