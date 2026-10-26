import { describe, expect, it } from 'vitest';

import { shortcutLabel } from './shortcuts';

describe('shortcut labels', () => {
  it('uses Ctrl on Linux and Windows', () => {
    expect(shortcutLabel('Mod+Shift+S', false)).toBe('Ctrl+Shift+S');
  });

  it('uses symbols on macOS', () => {
    expect(shortcutLabel('Mod+Shift+S', true)).toBe('⌘⇧S');
  });
});
