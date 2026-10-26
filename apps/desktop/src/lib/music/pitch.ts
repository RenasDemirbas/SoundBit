// Pitch naming for display. Musical rules live in Rust; this only labels MIDI numbers.

const NAMES = ['C', 'C#', 'D', 'D#', 'E', 'F', 'F#', 'G', 'G#', 'A', 'A#', 'B'] as const;

/** Pitch class (0 = C ... 11 = B) of a MIDI note. */
export function pitchClass(pitch: number): number {
  return ((pitch % 12) + 12) % 12;
}

/** Scientific pitch name: 60 is "C4". */
export function pitchName(pitch: number): string {
  return `${NAMES[pitchClass(pitch)] ?? '?'}${Math.floor(pitch / 12) - 1}`;
}

/** Name of a pitch class, e.g. 9 is "A". */
export function className(pitchClassValue: number): string {
  return NAMES[pitchClass(pitchClassValue)] ?? '?';
}

/** Whether the pitch is a black key on a piano. */
export function isBlackKey(pitch: number): boolean {
  return [1, 3, 6, 8, 10].includes(pitchClass(pitch));
}
