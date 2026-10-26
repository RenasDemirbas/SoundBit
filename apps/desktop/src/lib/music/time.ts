// Formatting playback times.

/** "m:ss" or, with `tenths`, "m:ss.t". Negative and missing values show as 0. */
export function formatTime(seconds: number, tenths = false): string {
  const safe = Number.isFinite(seconds) && seconds > 0 ? seconds : 0;
  const steps = tenths ? Math.floor(safe * 10) : Math.floor(safe);
  const whole = tenths ? Math.floor(steps / 10) : steps;
  const minutes = Math.floor(whole / 60);
  const rest = String(whole % 60).padStart(2, '0');
  return tenths ? `${minutes}:${rest}.${steps % 10}` : `${minutes}:${rest}`;
}
