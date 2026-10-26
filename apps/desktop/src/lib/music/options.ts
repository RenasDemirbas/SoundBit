// Choices offered in song settings, with display labels.

import type { ChipProfile, ScaleKind } from '../ipc/types';

export const CHIP_PROFILES: { value: ChipProfile; label: string }[] = [
  { value: 'free', label: 'Free' },
  { value: 'nes', label: 'NES' },
  { value: 'game_boy', label: 'Game Boy' },
  { value: 'c64', label: 'Commodore 64' },
  { value: 'genesis', label: 'Genesis' },
];

export const SCALE_KINDS: ScaleKind[] = [
  'chromatic',
  'major',
  'natural_minor',
  'harmonic_minor',
  'melodic_minor',
  'dorian',
  'phrygian',
  'lydian',
  'mixolydian',
  'locrian',
  'major_pentatonic',
  'minor_pentatonic',
  'blues',
  'whole_tone',
];

/** "natural_minor" → "Natural minor". */
export function scaleLabel(kind: ScaleKind): string {
  const words = kind.replaceAll('_', ' ');
  return words.charAt(0).toUpperCase() + words.slice(1);
}

export function profileLabel(profile: ChipProfile): string {
  return CHIP_PROFILES.find((option) => option.value === profile)?.label ?? profile;
}
