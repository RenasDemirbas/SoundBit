import { describe, expect, it } from 'vitest';

import { formatTime } from './time';

describe('formatTime', () => {
  it('formats minutes and seconds', () => {
    expect(formatTime(0)).toBe('0:00');
    expect(formatTime(33.5)).toBe('0:33');
    expect(formatTime(125.9)).toBe('2:05');
  });

  it('shows tenths when asked, without rounding up', () => {
    expect(formatTime(12.46, true)).toBe('0:12.4');
    expect(formatTime(59.99, true)).toBe('0:59.9');
    expect(formatTime(60, true)).toBe('1:00.0');
  });

  it('treats invalid values as zero', () => {
    expect(formatTime(-3)).toBe('0:00');
    expect(formatTime(Number.NaN, true)).toBe('0:00.0');
  });
});
