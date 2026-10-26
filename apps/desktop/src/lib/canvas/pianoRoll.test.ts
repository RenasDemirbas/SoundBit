import { describe, expect, it } from 'vitest';

import { MAX_STEP_WIDTH, MIN_STEP_WIDTH, pitchAtRow, rollGrid, rowOfPitch } from './pianoRoll';

describe('piano roll grid', () => {
  it('fills the available width', () => {
    expect(rollGrid(16, 52 + 2 + 16 * 40).cellWidth).toBe(40);
  });

  it('stays readable when narrow or wide', () => {
    expect(rollGrid(16, 100).cellWidth).toBe(MIN_STEP_WIDTH);
    expect(rollGrid(16, 10_000).cellWidth).toBe(MAX_STEP_WIDTH);
  });

  it('maps rows and pitches both ways', () => {
    expect(pitchAtRow(rowOfPitch(60))).toBe(60);
  });
});
