import { describe, expect, it } from 'vitest';

import { DEFAULT_THEME_ID, THEMES, findTheme } from './themes';

describe('themes', () => {
  it('falls back to the default theme', () => {
    expect(findTheme('nope').id).toBe(DEFAULT_THEME_ID);
    expect(findTheme(null).id).toBe(DEFAULT_THEME_ID);
    expect(findTheme('paper').id).toBe('paper');
  });

  it('defines the same variables and enough instrument colors everywhere', () => {
    const keys = Object.keys(THEMES[0]?.ui ?? {}).sort();
    for (const theme of THEMES) {
      expect(Object.keys(theme.ui).sort()).toEqual(keys);
      expect(theme.instruments.length).toBeGreaterThanOrEqual(8);
      // Clip fills derive translucent colors from these, so they must be #rrggbb.
      for (const color of theme.instruments) expect(color).toMatch(/^#[0-9a-f]{6}$/);
    }
  });

  it('has unique ids', () => {
    expect(new Set(THEMES.map((theme) => theme.id)).size).toBe(THEMES.length);
  });
});
