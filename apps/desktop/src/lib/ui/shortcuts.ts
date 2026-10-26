// Keyboard shortcut labels in the platform's style.

const isMac = typeof navigator !== 'undefined' && /Mac|iPhone|iPad/.test(navigator.userAgent);

/** "Ctrl+S" on Linux and Windows, "⌘S" on macOS. `keys` uses "Mod" for the command key. */
export function shortcutLabel(keys: string, mac = isMac): string {
  const parts = keys.split('+');
  if (mac) {
    const symbols: Record<string, string> = { Mod: '⌘', Shift: '⇧', Alt: '⌥' };
    return parts.map((part) => symbols[part] ?? part).join('');
  }
  return parts.map((part) => (part === 'Mod' ? 'Ctrl' : part)).join('+');
}
