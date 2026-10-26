import { describe, expect, it } from 'vitest';

import { className, isBlackKey, pitchClass, pitchName } from './pitch';

describe('pitch names', () => {
  it('matches scientific pitch notation used by the backend', () => {
    expect(pitchName(60)).toBe('C4');
    expect(pitchName(69)).toBe('A4');
    expect(pitchName(61)).toBe('C#4');
    expect(pitchName(0)).toBe('C-1');
    expect(pitchName(127)).toBe('G9');
  });

  it('names pitch classes', () => {
    expect(className(9)).toBe('A');
    expect(pitchClass(-1)).toBe(11);
  });

  it('knows black keys', () => {
    expect(isBlackKey(61)).toBe(true);
    expect(isBlackKey(64)).toBe(false);
  });
});
