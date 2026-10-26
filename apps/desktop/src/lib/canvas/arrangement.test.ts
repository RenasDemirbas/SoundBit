import { describe, expect, it } from 'vitest';

import { clipLook, fitText, withAlpha } from './arrangement';

/** Pretends every character is 10 pixels wide. */
const measure = (text: string): number => text.length * 10;

describe('clip labels', () => {
  it('keeps text that fits', () => {
    expect(fitText('Lead', 40, measure)).toBe('Lead');
  });

  it('shortens text with an ellipsis', () => {
    expect(fitText('Lead intro', 50, measure)).toBe('Lead…');
    expect(fitText('Chorus', 5, measure)).toBe('');
  });
});

describe('clip looks', () => {
  it('records the pitch range for the preview', () => {
    const look = clipLook('A', '#ffffff', [
      { pitch: 64, start: 0, length: 1, velocity: 100 },
      { pitch: 72, start: 4, length: 2, velocity: 100 },
    ]);
    expect([look.low, look.high]).toEqual([64, 72]);
  });

  it('has a neutral range without notes', () => {
    const look = clipLook('Empty', '#ffffff', []);
    expect(look.low).toBe(look.high);
  });
});

describe('colors', () => {
  it('adds an alpha channel', () => {
    expect(withAlpha('#5eead4', 0.5)).toBe('rgba(94, 234, 212, 0.5)');
  });
});
