// The active color theme: applied to CSS variables and read by the canvas editors.

import { instrumentColor } from '../canvas/grid';
import { DEFAULT_THEME_ID, findTheme, type Theme } from './themes';

const STORAGE_KEY = 'soundbit.theme';

/** Browser storage may be unavailable (private mode, blocked site data); themes still work. */
function storedThemeId(): string | null {
  try {
    return localStorage.getItem(STORAGE_KEY);
  } catch {
    return null;
  }
}

function storeThemeId(id: string): void {
  try {
    localStorage.setItem(STORAGE_KEY, id);
  } catch {
    // Remembering the choice is a convenience; ignore failures.
  }
}

class ThemeStore {
  current = $state<Theme>(findTheme(DEFAULT_THEME_ID));

  /** The color of the instrument at `index` in the current theme. */
  instrumentColor(index: number): string {
    return instrumentColor(index, this.current.instruments);
  }

  /** Restores the remembered theme. Call once at start-up. */
  init(): void {
    this.apply(findTheme(storedThemeId()));
  }

  select(id: string): void {
    this.apply(findTheme(id));
    storeThemeId(this.current.id);
  }

  private apply(theme: Theme): void {
    this.current = theme;
    const root = document.documentElement;
    for (const [name, value] of Object.entries(theme.ui)) {
      root.style.setProperty(`--${name}`, value);
    }
    root.style.colorScheme = theme.scheme;
  }
}

export const theme = new ThemeStore();
