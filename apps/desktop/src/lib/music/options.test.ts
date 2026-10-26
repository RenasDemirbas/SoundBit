import { describe, expect, it } from 'vitest';

import { profileLabel, scaleLabel } from './options';

describe('setting labels', () => {
  it('formats scale kinds', () => {
    expect(scaleLabel('natural_minor')).toBe('Natural minor');
    expect(scaleLabel('blues')).toBe('Blues');
  });

  it('names chip profiles', () => {
    expect(profileLabel('game_boy')).toBe('Game Boy');
  });
});
