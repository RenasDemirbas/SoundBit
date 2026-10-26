// Grouping the backend's preset library for menus.

import type { PresetInfo } from '../ipc/types';

export interface PresetGroup {
  category: string;
  presets: PresetInfo[];
}

/** Presets grouped by category, keeping the library's order of first appearance. */
export function groupPresets(presets: PresetInfo[]): PresetGroup[] {
  const groups = new Map<string, PresetInfo[]>();
  for (const preset of presets) {
    const group = groups.get(preset.category);
    if (group) group.push(preset);
    else groups.set(preset.category, [preset]);
  }
  return [...groups].map(([category, members]) => ({ category, presets: members }));
}

/** "fm_bass" category names as menu labels: "strings" -> "Strings". */
export function categoryLabel(category: string): string {
  return category.charAt(0).toUpperCase() + category.slice(1).replaceAll('_', ' ');
}
