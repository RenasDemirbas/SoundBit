// Slider mappings and labels for instrument sound shaping.

/** Filter cutoff range in hertz. */
export const CUTOFF_MIN = 20;
export const CUTOFF_MAX = 20_000;

/** Resolution of the logarithmic cutoff slider. */
export const CUTOFF_STEPS = 1000;

/** Cutoff for a slider position: logarithmic, so each octave gets the same travel. */
export function cutoffFromSlider(position: number): number {
  const fraction = Math.min(Math.max(position / CUTOFF_STEPS, 0), 1);
  return Math.round(CUTOFF_MIN * (CUTOFF_MAX / CUTOFF_MIN) ** fraction);
}

/** Slider position for a cutoff. */
export function sliderFromCutoff(hz: number): number {
  const clamped = Math.min(Math.max(hz, CUTOFF_MIN), CUTOFF_MAX);
  return Math.round((Math.log(clamped / CUTOFF_MIN) / Math.log(CUTOFF_MAX / CUTOFF_MIN)) * CUTOFF_STEPS);
}

/** "880 Hz" or "4.2 kHz". */
export function hertzLabel(hz: number): string {
  return hz < 1000 ? `${hz} Hz` : `${(hz / 1000).toFixed(1)} kHz`;
}

/** Vibrato rates are stored in tenths of a hertz. */
export const rateLabel = (tenths: number): string => `${(tenths / 10).toFixed(1)} Hz`;

/** "+12" or "-5" semitones. */
export const semitoneLabel = (semitones: number): string => (semitones > 0 ? `+${semitones}` : String(semitones));
