import { save } from "@tauri-apps/plugin-dialog";
import { useEffect, useState } from "react";
import { toast } from "sonner";

import { t } from "@/i18n/t";
import { type BundleFile, commands, type BundlePreview as Preview } from "@/ipc";
import { cn } from "@/lib/cn";
import { collectClientEnvironment } from "@/lib/environment";

import { Button } from "../ui/button";
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from "../ui/dialog";

/**
 * The preview step before a Diagnostics Bundle is written.
 *
 * **This is the most important part of the export.** wgm promises that nothing leaves
 * the machine without the user choosing. Handing them an opaque zip full of their
 * settings and file paths is not that promise kept — it is that promise asserted. So
 * every file is listed with its size, the small ones can be opened and read, and only
 * then is anything written.
 *
 * The dialog also says plainly that GitHub issue attachments are public and permanent:
 * the asset URL survives the issue being deleted.
 */
export function BundlePreviewDialog({
  open,
  onOpenChange,
}: {
  open: boolean;
  onOpenChange: (open: boolean) => void;
}) {
  const [preview, setPreview] = useState<Preview | null>(null);
  const [viewing, setViewing] = useState<BundleFile | null>(null);

  useEffect(() => {
    if (!open) {
      setPreview(null);
      setViewing(null);
      return;
    }

    void commands
      .diagnosticsPreview(collectClientEnvironment(), { correlationId: null, errorCode: null })
      .then(setPreview);
  }, [open]);

  const onSave = async () => {
    if (!preview) {
      return;
    }

    // The webview never touches a file: the picker returns a *path*, and Rust writes.
    const path = await save({
      defaultPath: preview.suggestedName,
      filters: [{ name: "Zip", extensions: ["zip"] }],
    });

    if (!path) {
      return;
    }

    const result = await commands.diagnosticsExport(path, collectClientEnvironment(), {
      correlationId: null,
      errorCode: null,
    });

    if (result.status === "ok") {
      toast.success(t("diagnostics.done"));
      onOpenChange(false);
    }
  };

  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogContent className="max-w-2xl">
        <DialogHeader>
          <DialogTitle>{t("diagnostics.title")}</DialogTitle>
          <DialogDescription>{t("diagnostics.intro")}</DialogDescription>
        </DialogHeader>

        {preview && !preview.redactionSafe && (
          <p role="alert" className="text-sm text-text">
            {t("diagnostics.unsafe")}
          </p>
        )}

        {viewing ? (
          <div className="flex min-h-0 flex-1 flex-col gap-2">
            <div className="flex items-center justify-between">
              <code className="text-xs text-text-muted">{viewing.name}</code>
              <Button size="small" variant="quiet" onClick={() => setViewing(null)}>
                {t("import.cancel")}
              </Button>
            </div>
            <pre
              data-selectable
              className={cn(
                "min-h-0 flex-1 overflow-auto rounded-sm border border-border bg-surface",
                "p-3 text-xs leading-4 text-text",
              )}
            >
              {viewing.preview}
            </pre>
          </div>
        ) : (
          <ul className="flex min-h-0 flex-1 flex-col divide-y divide-border overflow-y-auto">
            {preview?.files.map((file) => (
              <li key={file.name} className="flex items-center justify-between gap-4 py-2">
                <div className="flex min-w-0 flex-col">
                  <code className="truncate text-xs text-text">{file.name}</code>
                  <span className="text-xs text-text-muted">
                    {formatBytes(file.bytes)}
                    {file.truncated ? ` · ${t("diagnostics.truncated")}` : ""}
                  </span>
                </div>

                {file.preview !== null && (
                  <Button size="small" variant="quiet" onClick={() => setViewing(file)}>
                    {t("diagnostics.view")}
                  </Button>
                )}
              </li>
            ))}
          </ul>
        )}

        <p className="text-xs text-text-muted">{t("diagnostics.publicWarning")}</p>

        <DialogFooter>
          <Button onClick={() => onOpenChange(false)}>{t("diagnostics.cancel")}</Button>
          <Button
            variant="primary"
            disabled={!preview?.redactionSafe}
            onClick={() => void onSave()}
          >
            {t("diagnostics.save")}
          </Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  );
}

function formatBytes(bytes: number): string {
  if (bytes < 1024) {
    return `${bytes} B`;
  }
  if (bytes < 1024 * 1024) {
    return `${Math.round(bytes / 1024)} KB`;
  }
  return `${(bytes / (1024 * 1024)).toFixed(1)} MB`;
}
