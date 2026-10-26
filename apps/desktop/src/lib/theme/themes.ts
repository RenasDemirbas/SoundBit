// Color themes. Each theme sets the CSS variables used by components and the colors the
// canvas editors draw with, so both always match.

/** Colors used by the canvas editors (piano roll, arrangement). */
export interface CanvasColors {
  background: string;
  row: string;
  rowAlt: string;
  whiteRow: string;
  blackRow: string;
  scaleRow: string;
  line: string;
  lineStrong: string;
  label: string;
  keyWhite: string;
  keyBlack: string;
  keyWhiteText: string;
  playhead: string;
  playheadFill: string;
  selection: string;
  noteEdge: string;
  loopTint: string;
}

export interface Theme {
  id: string;
  name: string;
  scheme: 'dark' | 'light';
  /** CSS custom properties, without the leading `--`. */
  ui: Record<string, string>;
  canvas: CanvasColors;
  /** Instrument colors, cycled in instrument order. */
  instruments: readonly string[];
}

const ui = (
  base: { bg: string; panel: string; raised: string; border: string; strong: string },
  text: { text: string; muted: string },
  accents: { accent: string; accent2: string; danger: string },
  extra: { ruler: string; loop: string; shadow: string },
): Record<string, string> => ({
  bg: base.bg,
  panel: base.panel,
  'panel-raised': base.raised,
  border: base.border,
  'border-strong': base.strong,
  text: text.text,
  muted: text.muted,
  accent: accents.accent,
  'accent-2': accents.accent2,
  danger: accents.danger,
  'ruler-bg': extra.ruler,
  'loop-bg': extra.loop,
  shadow: extra.shadow,
  'grid-line': base.border,
  'grid-line-strong': base.strong,
});

const DARK_SHADOW = '0 12px 32px rgba(0, 0, 0, 0.45)';

export const THEMES: readonly Theme[] = [
  {
    id: 'graphite',
    name: 'Graphite',
    scheme: 'dark',
    ui: ui(
      { bg: '#0d0f13', panel: '#15181e', raised: '#1e222a', border: '#2a2f39', strong: '#3a4150' },
      { text: '#e7e9ee', muted: '#8c93a1' },
      { accent: '#3dd6c6', accent2: '#f5c451', danger: '#f87171' },
      { ruler: '#101317', loop: 'rgba(245, 196, 81, 0.22)', shadow: DARK_SHADOW },
    ),
    canvas: {
      background: '#111419', row: '#161a20', rowAlt: '#14171d', whiteRow: '#191d24', blackRow: '#14171d',
      scaleRow: '#1b2530', line: '#222731', lineStrong: '#343b48', label: '#8c93a1', keyWhite: '#d9dde5',
      keyBlack: '#0b0d10', keyWhiteText: '#1a1d23', playhead: '#3dd6c6', playheadFill: 'rgba(61, 214, 198, 0.16)',
      selection: '#ffffff', noteEdge: 'rgba(0, 0, 0, 0.45)', loopTint: 'rgba(245, 196, 81, 0.06)',
    },
    instruments: ['#3dd6c6', '#f5c451', '#f472b6', '#60a5fa', '#a3e635', '#fb923c', '#a78bfa', '#f87171'],
  },
  {
    id: 'midnight',
    name: 'Midnight',
    scheme: 'dark',
    ui: ui(
      { bg: '#0f0c1a', panel: '#18122b', raised: '#221a3a', border: '#2f2650', strong: '#40385f' },
      { text: '#e8e3ff', muted: '#9d94c4' },
      { accent: '#5eead4', accent2: '#fcd34d', danger: '#f87171' },
      { ruler: '#0f0c1a', loop: 'rgba(252, 211, 77, 0.25)', shadow: DARK_SHADOW },
    ),
    canvas: {
      background: '#141021', row: '#1a1430', rowAlt: '#17122a', whiteRow: '#1d1730', blackRow: '#171226',
      scaleRow: '#232043', line: '#251f3b', lineStrong: '#3a3258', label: '#9d94c4', keyWhite: '#d9d3f5',
      keyBlack: '#0c0915', keyWhiteText: '#1b1430', playhead: '#5eead4', playheadFill: 'rgba(94, 234, 212, 0.18)',
      selection: '#ffffff', noteEdge: 'rgba(0, 0, 0, 0.5)', loopTint: 'rgba(252, 211, 77, 0.05)',
    },
    instruments: ['#5eead4', '#fcd34d', '#f472b6', '#93c5fd', '#a3e635', '#fb923c', '#c4b5fd', '#f87171'],
  },
  {
    id: 'game-boy',
    name: 'Game Boy',
    scheme: 'dark',
    ui: ui(
      { bg: '#0f1a0c', panel: '#162413', raised: '#1f3219', border: '#2e4a26', strong: '#446b37' },
      { text: '#e3f5c8', muted: '#92b07e' },
      { accent: '#9bd83c', accent2: '#e8f86c', danger: '#ff7a59' },
      { ruler: '#0d170a', loop: 'rgba(232, 248, 108, 0.2)', shadow: DARK_SHADOW },
    ),
    canvas: {
      background: '#12200e', row: '#172812', rowAlt: '#142310', whiteRow: '#1a2c15', blackRow: '#142310',
      scaleRow: '#22391b', line: '#24391e', lineStrong: '#3a5a2f', label: '#92b07e', keyWhite: '#c4dc9a',
      keyBlack: '#0b1408', keyWhiteText: '#1a2c15', playhead: '#9bd83c', playheadFill: 'rgba(155, 216, 60, 0.16)',
      selection: '#f0ffd0', noteEdge: 'rgba(0, 0, 0, 0.45)', loopTint: 'rgba(232, 248, 108, 0.06)',
    },
    instruments: ['#9bd83c', '#e8f86c', '#5fc4a0', '#c4e08a', '#7fb36a', '#d8c46a', '#a6d8b8', '#ff9a6a'],
  },
  {
    id: 'synthwave',
    name: 'Synthwave',
    scheme: 'dark',
    ui: ui(
      { bg: '#110620', panel: '#190b2d', raised: '#25123f', border: '#3b1d60', strong: '#57308a' },
      { text: '#f7ecff', muted: '#b39bd6' },
      { accent: '#ff4fd8', accent2: '#2ee6ff', danger: '#ff6b6b' },
      { ruler: '#0e051b', loop: 'rgba(46, 230, 255, 0.2)', shadow: DARK_SHADOW },
    ),
    canvas: {
      background: '#150828', row: '#1b0c33', rowAlt: '#180a2e', whiteRow: '#1e0e38', blackRow: '#170a2c',
      scaleRow: '#2a1250', line: '#2c1648', lineStrong: '#4b2678', label: '#b39bd6', keyWhite: '#e6d4ff',
      keyBlack: '#0c0418', keyWhiteText: '#1e0e38', playhead: '#ff4fd8', playheadFill: 'rgba(255, 79, 216, 0.16)',
      selection: '#ffffff', noteEdge: 'rgba(0, 0, 0, 0.45)', loopTint: 'rgba(46, 230, 255, 0.06)',
    },
    instruments: ['#ff4fd8', '#2ee6ff', '#ffb347', '#b388ff', '#5cff9d', '#ff6f91', '#ffe66d', '#7aa2ff'],
  },
  {
    id: 'paper',
    name: 'Paper (light)',
    scheme: 'light',
    ui: ui(
      { bg: '#eef0f4', panel: '#ffffff', raised: '#f1f3f7', border: '#d8dde6', strong: '#b9c1ce' },
      { text: '#1d2129', muted: '#6b7280' },
      { accent: '#0d9488', accent2: '#c27c0e', danger: '#dc2626' },
      { ruler: '#f5f6f9', loop: 'rgba(194, 124, 14, 0.18)', shadow: '0 12px 32px rgba(15, 23, 42, 0.16)' },
    ),
    canvas: {
      background: '#fbfcfd', row: '#ffffff', rowAlt: '#f6f7fa', whiteRow: '#ffffff', blackRow: '#f1f3f6',
      scaleRow: '#e3f3f1', line: '#e6e9ef', lineStrong: '#c5ccd8', label: '#6b7280', keyWhite: '#ffffff',
      keyBlack: '#2d333d', keyWhiteText: '#4b5563', playhead: '#0d9488', playheadFill: 'rgba(13, 148, 136, 0.12)',
      selection: '#1d2129', noteEdge: 'rgba(0, 0, 0, 0.25)', loopTint: 'rgba(194, 124, 14, 0.07)',
    },
    instruments: ['#0d9488', '#c27c0e', '#db2777', '#2563eb', '#65a30d', '#ea580c', '#7c3aed', '#dc2626'],
  },
];

export const DEFAULT_THEME_ID = 'graphite';

/** The theme with this id, or the default. */
export function findTheme(id: string | null | undefined): Theme {
  return THEMES.find((theme) => theme.id === id) ?? THEMES.find((theme) => theme.id === DEFAULT_THEME_ID) ?? (THEMES[0] as Theme);
}
