<script lang="ts">
  import type { Instrument, InstrumentChanges, PitchEnvelope, Unison, Vibrato } from '../ipc/types';
  import { rateLabel, semitoneLabel } from '../music/shaping';
  import Slider from '../ui/Slider.svelte';

  let { instrument, onchange }: { instrument: Instrument; onchange: (changes: InstrumentChanges) => void } = $props();

  const ms = (value: number): string => `${value} ms`;
  const cents = (value: number): string => `${value} ct`;
  const pitch = (change: Partial<PitchEnvelope>): void =>
    onchange({ pitch_envelope: { ...instrument.pitch_envelope, ...change } });
  const vibrato = (change: Partial<Vibrato>): void => onchange({ vibrato: { ...instrument.vibrato, ...change } });
  const unison = (change: Partial<Unison>): void => onchange({ unison: { ...instrument.unison, ...change } });
</script>

<h4>Pitch sweep</h4>
<Slider label="Start" min={-48} max={48} value={instrument.pitch_envelope.semitones} format={semitoneLabel}
  title="Semitones away from the note at its start; glides to the note (kicks, toms, lasers)"
  onchange={(semitones) => pitch({ semitones })} />
<Slider label="Glide" min={0} max={1000} step={5} value={instrument.pitch_envelope.time} format={ms}
  onchange={(time) => pitch({ time })} />

<h4>Vibrato</h4>
<Slider label="Depth" min={0} max={100} value={instrument.vibrato.depth} format={cents}
  title="Pitch wobble either side of the note" onchange={(depth) => vibrato({ depth })} />
<Slider label="Rate" min={1} max={200} value={instrument.vibrato.rate} format={rateLabel}
  onchange={(rate) => vibrato({ rate })} />
<Slider label="Delay" min={0} max={2000} step={10} value={instrument.vibrato.delay} format={ms}
  title="How long a note plays before the vibrato starts" onchange={(delay) => vibrato({ delay })} />

<h4>Unison</h4>
<Slider label="Voices" min={1} max={4} value={instrument.unison.voices}
  title="Detuned copies of the voice: thicker strings, choirs and leads" onchange={(voices) => unison({ voices })} />
<Slider label="Detune" min={0} max={50} value={instrument.unison.detune} format={cents}
  onchange={(detune) => unison({ detune })} />

<style>
  h4 {
    margin: 4px 0 0;
    font-size: 10px;
    font-weight: 600;
    letter-spacing: 0.06em;
    text-transform: uppercase;
    color: var(--muted);
  }
</style>
