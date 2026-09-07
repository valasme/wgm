import type { Settings } from "@/ipc";
import { useSettingsStore } from "@/stores/settings-store";

import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from "../ui/select";
import { SettingRow } from "./setting-row";

/**
 * A one-of-many settings row, applied instantly.
 *
 * Same contract as the switch: optimistic, confirmed by Rust, rolled back and explained
 * on the row if the write fails.
 */
export function SettingSelect<Value extends string>({
  id,
  label,
  description,
  value,
  options,
  optimistic,
  commit,
}: {
  id: string;
  label: string;
  description?: string;
  value: Value;
  options: readonly { value: Value; label: string }[];
  optimistic: (settings: Settings, next: Value) => Settings;
  commit: (next: Value) => Promise<Settings>;
}) {
  const apply = useSettingsStore((state) => state.apply);

  return (
    <SettingRow
      id={id}
      label={label}
      description={description}
      control={({ labelId, describedBy }) => (
        <Select
          value={value}
          onValueChange={(next) =>
            void apply(
              id,
              (settings) => optimistic(settings, next as Value),
              () => commit(next as Value),
            )
          }
        >
          <SelectTrigger aria-labelledby={labelId} aria-describedby={describedBy}>
            <SelectValue />
          </SelectTrigger>
          <SelectContent>
            {options.map((option) => (
              <SelectItem key={option.value} value={option.value}>
                {option.label}
              </SelectItem>
            ))}
          </SelectContent>
        </Select>
      )}
    />
  );
}
