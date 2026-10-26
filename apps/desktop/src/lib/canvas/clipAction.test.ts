import { describe, expect, it } from 'vitest';

import { clipAction } from './clipAction';

describe('arrangement clicks', () => {
  it('places the selected pattern in empty or other bars', () => {
    expect(clipAction(null, 3, false)).toEqual({ kind: 'place', pattern: 3 });
    expect(clipAction(5, 3, false)).toEqual({ kind: 'place', pattern: 3 });
  });

  it('clears a bar that already holds the selected pattern', () => {
    expect(clipAction(3, 3, false)).toEqual({ kind: 'clear' });
  });

  it('selects with alt or when nothing is selected', () => {
    expect(clipAction(5, 3, true)).toEqual({ kind: 'select', pattern: 5 });
    expect(clipAction(5, null, false)).toEqual({ kind: 'select', pattern: 5 });
    expect(clipAction(null, null, false)).toEqual({ kind: 'none' });
  });
});
