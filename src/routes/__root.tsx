import { createRootRoute, Link, Outlet } from "@tanstack/react-router";
import { Toaster } from "sonner";
import { RouteError } from "@/components/errors/route-error";
import { PagePlaceholder } from "@/components/states/page-placeholder";
import { Button } from "@/components/ui/button";
import { TooltipProvider } from "@/components/ui/tooltip";
import { t } from "@/i18n/t";

export const Route = createRootRoute({
  component: RootLayout,
  errorComponent: RouteError,
  notFoundComponent: NotFound,
});

function RootLayout() {
  return (
    <TooltipProvider delayDuration={400} skipDelayDuration={200}>
      <Outlet />

      {/*
        Bottom-right. Top-right would collide with the Window Controls, and a toast
        that covers the close button is a toast that traps the user.
      */}
      <Toaster
        position="bottom-right"
        toastOptions={{
          classNames: {
            toast: "border border-border bg-bg text-text text-sm rounded-md",
            description: "text-text-muted",
          },
        }}
      />
    </TooltipProvider>
  );
}

function NotFound() {
  return (
    <PagePlaceholder
      title="route.notFound.title"
      heading="route.notFound.heading"
      action={
        <Button asChild variant="primary">
          <Link to="/search">{t("route.notFound.action")}</Link>
        </Button>
      }
    />
  );
}
