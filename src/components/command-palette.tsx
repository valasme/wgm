import { useNavigate } from "@tanstack/react-router";
import { Command } from "cmdk";
import {
  BoxIcon,
  HardDriveDownloadIcon,
  PaletteIcon,
  RefreshCwIcon,
  SearchIcon,
  SettingsIcon,
} from "lucide-react";
import type { ComponentType } from "react";

import { t } from "@/i18n/t";
import { type ColorSchemePreference, commands } from "@/ipc";
import { cn } from "@/lib/cn";
import { useSettingsStore } from "@/stores/settings-store";
import { COLOR_SCHEME_LABELS, COLOR_SCHEMES } from "@/theme/accents";

import { Dialog, DialogContent, DialogDescription, DialogTitle } from "./ui/dialog";

interface PaletteEntry {
  id: string;
  label: string;
  icon: ComponentType<{ className?: string }>;
  run: () => void;
}

/**
 * The Command Palette.
 *
 * `Ctrl+K`, and also reachable by pointer from the Sidebar pill — a keyboard-only route
 * to a keyboard feature helps nobody who does not already know it.
 *
 * It navigates routes, switches Color Scheme, and jumps to any settings page. Nothing
 * here searches packages: wgm does not invoke winget yet, and a palette entry that does
 * nothing is a dead control.
 */
export function CommandPalette({
  open,
  onOpenChange,
}: {
  open: boolean;
  onOpenChange: (open: boolean) => void;
}) {
  const navigate = useNavigate();
  const applySetting = useSettingsStore((state) => state.apply);

  const go = (to: string) => {
    onOpenChange(false);
    void navigate({ to });
  };

  const routes: PaletteEntry[] = [
    { id: "search", label: t("nav.search"), icon: SearchIcon, run: () => go("/search") },
    { id: "kits", label: t("nav.kits"), icon: BoxIcon, run: () => go("/kits") },
    {
      id: "installed",
      label: t("nav.installed"),
      icon: HardDriveDownloadIcon,
      run: () => go("/installed"),
    },
    { id: "updates", label: t("nav.updates"), icon: RefreshCwIcon, run: () => go("/updates") },
  ];

  const settingsPages: PaletteEntry[] = [
    {
      id: "settings-appearance",
      label: t("settings.appearance.title"),
      icon: SettingsIcon,
      run: () => go("/settings/appearance"),
    },
    {
      id: "settings-general",
      label: t("settings.general.title"),
      icon: SettingsIcon,
      run: () => go("/settings/general"),
    },
    {
      id: "settings-advanced",
      label: t("settings.advanced.title"),
      icon: SettingsIcon,
      run: () => go("/settings/advanced"),
    },
    {
      id: "settings-about",
      label: t("settings.about.title"),
      icon: SettingsIcon,
      run: () => go("/settings/about"),
    },
  ];

  const setScheme = (scheme: ColorSchemePreference) => {
    onOpenChange(false);
    void applySetting(
      "appearance.colorScheme",
      (settings) => ({
        ...settings,
        appearance: { ...settings.appearance, colorScheme: scheme },
      }),
      async () => {
        const result = await commands.settingsSetColorScheme(scheme);
        if (result.status === "error") {
          throw result.error;
        }
        return result.data;
      },
    );
  };

  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogContent className="max-w-xl gap-0 p-0">
        <DialogTitle className="sr-only">{t("palette.open")}</DialogTitle>
        <DialogDescription className="sr-only">{t("palette.placeholder")}</DialogDescription>

        <Command label={t("palette.open")} className="flex flex-col">
          <Command.Input
            placeholder={t("palette.placeholder")}
            className={cn(
              "h-11 w-full border-b border-border bg-transparent px-4",
              "text-sm text-text outline-none placeholder:text-text-muted",
            )}
          />

          <Command.List className="max-h-80 overflow-y-auto p-1">
            <Command.Empty className="p-4 text-center text-sm text-text-muted">
              {t("palette.empty")}
            </Command.Empty>

            <Group heading={t("palette.groupNavigate")} items={routes} />
            <Group heading={t("palette.groupSettings")} items={settingsPages} />

            <Command.Group
              heading={t("palette.groupAppearance")}
              className="px-2 py-1 text-xs text-text-muted [&_[cmdk-group-items]]:mt-1"
            >
              {COLOR_SCHEMES.map((scheme) => (
                <Item
                  key={scheme}
                  icon={PaletteIcon}
                  label={t("palette.setColorScheme", {
                    scheme: t(COLOR_SCHEME_LABELS[scheme]),
                  })}
                  onSelect={() => setScheme(scheme)}
                />
              ))}
            </Command.Group>
          </Command.List>
        </Command>
      </DialogContent>
    </Dialog>
  );
}

function Group({ heading, items }: { heading: string; items: PaletteEntry[] }) {
  return (
    <Command.Group
      heading={heading}
      className="px-2 py-1 text-xs text-text-muted [&_[cmdk-group-items]]:mt-1"
    >
      {items.map((item) => (
        <Item key={item.id} icon={item.icon} label={item.label} onSelect={item.run} />
      ))}
    </Command.Group>
  );
}

function Item({
  icon: Icon,
  label,
  onSelect,
}: {
  icon: ComponentType<{ className?: string }>;
  label: string;
  onSelect: () => void;
}) {
  return (
    <Command.Item
      value={label}
      onSelect={onSelect}
      className={cn(
        "flex h-8 cursor-default items-center gap-2 rounded-sm px-2",
        "text-sm text-text",
        "data-[selected=true]:bg-surface",
      )}
    >
      <Icon className="size-4 shrink-0 text-text-muted" aria-hidden="true" />
      {label}
    </Command.Item>
  );
}
