import type { Settings } from "@/ipc";
import { useSettingsStore } from "@/stores/settings-store";

import { Switch } from "../ui/switch";
import { SettingRow } from "./setting-row";

/**
 * A boolean settings row, applied instantly.
 *
 * The switch moves now, Rust confirms, and on failure the store puts it back and the
 * row shows why. No Save button, no dirty state, no navigation guards — and no
 * `disabled` while the write is in flight either, because a control that freezes on
 * every toggle is worse than one that occasionally comes back.
 */
export function SettingSwitch({
  id,
  label,
  description,
  value,
  optimistic,
  commit,
}: {
  id: string;
  label: string;
  description?: string;
  value: boolean;
  optimistic: (settings: Settings, next: boolean) => Settings;
  commit: (next: boolean) => Promise<Settings>;
}) {
  const apply = useSettingsStore((state) => state.apply);

  return (
    <SettingRow
      id={id}
      label={label}
      description={description}
      control={({ labelId, describedBy }) => (
        <Switch
          checked={value}
          aria-labelledby={labelId}
          aria-describedby={describedBy}
          onCheckedChange={(next) =>
            void apply(
              id,
              (settings) => optimistic(settings, next),
              () => commit(next),
            )
          }
        />
      )}
    />
  );
}
