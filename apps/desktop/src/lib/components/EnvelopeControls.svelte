<script lang="ts">
  import type { Envelope } from '../ipc/types';
  import Slider from '../ui/Slider.svelte';

  let { envelope, onchange }: { envelope: Envelope; onchange: (envelope: Envelope) => void } = $props();

  const ms = (value: number): string => `${value} ms`;
  const set = (change: Partial<Envelope>): void => onchange({ ...envelope, ...change });
</script>

<Slider label="Attack" min={0} max={2000} step={5} value={envelope.attack} format={ms}
  title="Time to rise to full level" onchange={(attack) => set({ attack })} />
<Slider label="Decay" min={0} max={3000} step={5} value={envelope.decay} format={ms}
  title="Time to fall to the sustain level" onchange={(decay) => set({ decay })} />
<Slider label="Sustain" min={0} max={100} value={envelope.sustain} format={(v) => `${v}%`}
  title="Level held while the note is on" onchange={(sustain) => set({ sustain })} />
<Slider label="Release" min={0} max={4000} step={5} value={envelope.release} format={ms}
  title="Time to fade out after the note ends" onchange={(release) => set({ release })} />
