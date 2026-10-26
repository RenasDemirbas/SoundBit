<script lang="ts">
  import type { Filter, FilterMode } from '../ipc/types';
  import { CUTOFF_STEPS, cutoffFromSlider, hertzLabel, semitoneLabel, sliderFromCutoff } from '../music/shaping';
  import Segmented from '../ui/Segmented.svelte';
  import Slider from '../ui/Slider.svelte';

  let { filter, onchange }: { filter: Filter; onchange: (filter: Filter) => void } = $props();

  const MODES: { value: FilterMode; label: string; title: string }[] = [
    { value: 'off', label: 'Off', title: 'No filtering' },
    { value: 'low_pass', label: 'LP', title: 'Low-pass: softer, darker' },
    { value: 'high_pass', label: 'HP', title: 'High-pass: thinner, brighter' },
    { value: 'band_pass', label: 'BP', title: 'Band-pass: nasal' },
  ];

  const set = (change: Partial<Filter>): void => onchange({ ...filter, ...change });
</script>

<div class="field">Filter
  <Segmented label="Filter type" options={MODES} value={filter.mode} onchange={(mode) => set({ mode })} />
</div>
{#if filter.mode !== 'off'}
  <Slider label="Cutoff" min={0} max={CUTOFF_STEPS} value={sliderFromCutoff(filter.cutoff)}
    format={(position) => hertzLabel(cutoffFromSlider(position))}
    onchange={(position) => set({ cutoff: cutoffFromSlider(position) })} />
  <Slider label="Resonance" min={0} max={100} value={filter.resonance} format={(v) => `${v}%`}
    onchange={(resonance) => set({ resonance })} />
  <Slider label="Env amount" min={-72} max={72} value={filter.envelope_depth} format={semitoneLabel}
    title="Semitones the cutoff moves with the amplitude envelope"
    onchange={(envelope_depth) => set({ envelope_depth })} />
{/if}

<style>
  .field {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 8px;
    font-size: 11px;
    color: var(--muted);
  }
</style>
