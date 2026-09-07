import { createFileRoute, Link } from "@tanstack/react-router";
import { HardDriveDownloadIcon } from "lucide-react";

import { PagePlaceholder } from "@/components/states/page-placeholder";
import { Button } from "@/components/ui/button";
import { t } from "@/i18n/t";

export const Route = createFileRoute("/_app/installed")({
  component: InstalledPage,
});

function InstalledPage() {
  return (
    <PagePlaceholder
      title="route.installed.title"
      heading="route.installed.heading"
      body="route.installed.body"
      icon={<HardDriveDownloadIcon className="size-8" aria-hidden="true" />}
      action={
        <Button asChild variant="primary">
          <Link to="/search">{t("route.installed.action")}</Link>
        </Button>
      }
    />
  );
}
