// What a click on an arrangement cell does.

export type ClipAction =
  | { kind: 'place'; pattern: number }
  | { kind: 'clear' }
  | { kind: 'select'; pattern: number }
  | { kind: 'none' };

/**
 * Alt+click selects the clicked clip. Otherwise clicking places the selected pattern, clicking a
 * bar that already holds it clears the bar, and with nothing selected a click selects the clip.
 */
export function clipAction(current: number | null, selected: number | null, alt: boolean): ClipAction {
  if (alt && current !== null) return { kind: 'select', pattern: current };
  if (current !== null && current === selected) return { kind: 'clear' };
  if (selected !== null) return { kind: 'place', pattern: selected };
  if (current !== null) return { kind: 'select', pattern: current };
  return { kind: 'none' };
}
