import { describe, expect, it } from 'vitest';

import { CUTOFF_MAX, CUTOFF_MIN, CUTOFF_STEPS, cutoffFromSlider, hertzLabel, sliderFromCutoff } from './shaping';

describe('cutoff slider', () => {
  it('spans the full range', () => {
    expect(cutoffFromSlider(0)).toBe(CUTOFF_MIN);
    expect(cutoffFromSlider(CUTOFF_STEPS)).toBe(CUTOFF_MAX);
  });

  it('round-trips through slider positions', () => {
    for (const hz of [20, 100, 440, 2000, 12_000]) {
      const back = cutoffFromSlider(sliderFromCutoff(hz));
      expect(Math.abs(back - hz) / hz).toBeLessThan(0.01);
    }
  });

  it('formats frequencies', () => {
    expect(hertzLabel(880)).toBe('880 Hz');
    expect(hertzLabel(4200)).toBe('4.2 kHz');
  });
});
