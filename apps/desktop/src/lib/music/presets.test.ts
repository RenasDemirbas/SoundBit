import { describe, expect, it } from 'vitest';

import type { PresetInfo } from '../ipc/types';
import { categoryLabel, groupPresets } from './presets';

const preset = (id: string, category: string): PresetInfo => ({
  id,
  name: id,
  category,
  description: '',
  voice: 'sine',
});

describe('groupPresets', () => {
  it('groups by category in order of first appearance', () => {
    const groups = groupPresets([preset('a', 'strings'), preset('b', 'drums'), preset('c', 'strings')]);
    expect(groups.map((g) => g.category)).toEqual(['strings', 'drums']);
    expect(groups[0]?.presets.map((p) => p.id)).toEqual(['a', 'c']);
  });

  it('labels categories', () => {
    expect(categoryLabel('woodwinds')).toBe('Woodwinds');
    expect(categoryLabel('sound_fx')).toBe('Sound fx');
  });
});
