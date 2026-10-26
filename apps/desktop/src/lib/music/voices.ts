// Default parameters per voice type. Ready-made instruments are presets loaded from the
// backend (`assets/presets/`).

import type { FmOperator, Voice, VoiceKind } from '../ipc/types';

const carrier = (multiple: number, totalLevel = 0): FmOperator => ({
  multiple,
  detune: 0,
  total_level: totalLevel,
  attack_rate: 31,
  decay_rate: 6,
  sustain_rate: 0,
  sustain_level: 4,
  release_rate: 7,
});

const silent: FmOperator = { ...carrier(1), total_level: 127 };

/** A Game Boy-style triangle: 0 to 15 and back. */
const triangleTable = Array.from({ length: 32 }, (_, i) => (i < 16 ? i : 31 - i));

export const VOICE_KINDS: VoiceKind[] = ['pulse', 'triangle', 'sawtooth', 'sine', 'noise', 'wavetable', 'fm'];

/** Default parameters when switching an instrument to another voice type. */
export function defaultVoice(kind: VoiceKind): Voice {
  switch (kind) {
    case 'pulse':
      return { type: 'pulse', duty: 'quarter' };
    case 'noise':
      return { type: 'noise', mode: 'long' };
    case 'wavetable':
      return { type: 'wavetable', samples: triangleTable };
    case 'fm':
      return { type: 'fm', algorithm: 7, feedback: 0, operators: [carrier(1), silent, silent, silent] };
    default:
      return { type: kind };
  }
}
