import { createFileRoute, Link } from "@tanstack/react-router";

import { PagePlaceholder } from "@/components/states/page-placeholder";
import { Button } from "@/components/ui/button";
import { t } from "@/i18n/t";

export const Route = createFileRoute("/_app/kits/$kitId")({
  component: KitPage,
});

/**
 * No Kit is ever found: there is no `.wgmkit` reader yet. The not-found state is the
 * honest one to render, and it is the state this route will spend most of its life in
 * once there is a reader — a link someone shared for a kit they never published.
 */
function KitPage() {
  return (
    <PagePlaceholder
      title="route.kit.title"
      heading="route.kit.heading"
      body="route.kit.body"
      action={
        <Button asChild>
          <Link to="/kits">{t("route.kit.action")}</Link>
        </Button>
      }
    />
  );
}
