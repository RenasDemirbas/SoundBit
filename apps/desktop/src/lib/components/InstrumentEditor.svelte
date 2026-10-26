<script lang="ts">
  import type { DutyCycle, Instrument, InstrumentChanges, NoiseMode, VoiceKind } from '../ipc/types';
  import { VOICE_KINDS, defaultVoice } from '../music/voices';
  import { editor } from '../stores/editor.svelte';
  import Icon from '../ui/Icon.svelte';
  import Segmented from '../ui/Segmented.svelte';
  import Slider from '../ui/Slider.svelte';
  import EnvelopeControls from './EnvelopeControls.svelte';
  import FilterControls from './FilterControls.svelte';
  import ModulationControls from './ModulationControls.svelte';

  let { instrument }: { instrument: Instrument } = $props();

  type Input = Event & { currentTarget: HTMLInputElement };

  const DUTIES: { value: DutyCycle; label: string; title: string }[] = [
    { value: 'eighth', label: '12.5', title: '12.5 %: thin and nasal' },
    { value: 'quarter', label: '25', title: '25 %: the classic lead' },
    { value: 'half', label: '50', title: '50 %: hollow square' },
    { value: 'three_quarters', label: '75', title: '75 %' },
  ];
  const MODES: { value: NoiseMode; label: string; title: string }[] = [
    { value: 'long', label: 'Hiss', title: 'Long sequence: snares, hi-hats' },
    { value: 'short', label: 'Metal', title: 'Short sequence: metallic, tonal' },
  ];

  const act = (changes: InstrumentChanges): void =>
    void editor.act({ kind: 'update_instrument', id: instrument.id, ...changes });

  function setName(event: Input): void {
    const name = event.currentTarget.value.trim();
    if (name && name !== instrument.name) act({ name });
  }

  /** "L40", "C" or "R25". */
  const panLabel = (pan: number): string => (pan === 0 ? 'C' : `${pan < 0 ? 'L' : 'R'}${Math.abs(pan)}`);
</script>

<div class="editor">
  <div class="name row">
    <input value={instrument.name} onchange={setName} aria-label="Instrument name" spellcheck="false" />
    <button
      class="icon-button"
      title="Remove instrument"
      aria-label="Remove instrument"
      onclick={() => void editor.act({ kind: 'remove_instrument', id: instrument.id })}><Icon name="trash" /></button
    >
  </div>

  <label class="field">Voice
    <select
      value={instrument.voice.type}
      onchange={(event) => act({ voice: defaultVoice(event.currentTarget.value as VoiceKind) })}
    >
      {#each VOICE_KINDS as kind (kind)}<option value={kind}>{kind}</option>{/each}
    </select>
  </label>

  {#if instrument.voice.type === 'pulse'}
    <div class="field">Duty
      <Segmented label="Duty cycle" options={DUTIES} value={instrument.voice.duty}
        onchange={(duty) => act({ voice: { type: 'pulse', duty } })} />
    </div>
  {:else if instrument.voice.type === 'noise'}
    <div class="field">Noise
      <Segmented label="Noise mode" options={MODES} value={instrument.voice.mode}
        onchange={(mode) => act({ voice: { type: 'noise', mode } })} />
    </div>
  {/if}

  <Slider label="Volume" min={0} max={100} value={instrument.volume} onchange={(volume) => act({ volume })} />
  <Slider label="Pan" min={-100} max={100} value={instrument.pan} format={panLabel} onchange={(pan) => act({ pan })} />
  <Slider label="Reverb" min={0} max={100} value={instrument.reverb_send} format={(v) => `${v}%`}
    title="Share of the sound sent into the song's reverb room" onchange={(reverb_send) => act({ reverb_send })} />

  <details>
    <summary>Envelope{instrument.voice.type === 'fm' ? ' (filter only)' : ''}</summary>
    <EnvelopeControls envelope={instrument.envelope} onchange={(envelope) => act({ envelope })} />
  </details>
  <details>
    <summary>Filter</summary>
    <FilterControls filter={instrument.filter} onchange={(filter) => act({ filter })} />
  </details>
  <details>
    <summary>Pitch &amp; unison</summary>
    <ModulationControls {instrument} onchange={act} />
  </details>
</div>

<style>
  .editor {
    display: flex;
    flex-direction: column;
    gap: 8px;
    margin-top: 8px;
    padding-top: 10px;
    border-top: 1px solid var(--border);
  }

  .name {
    flex-wrap: nowrap;
  }

  .name input {
    flex: 1;
    min-width: 0;
    border-color: transparent;
    background: transparent;
    font-weight: 600;
  }

  .name input:focus {
    border-color: var(--accent);
    background: var(--bg);
  }

  .field {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 8px;
    font-size: 11px;
    color: var(--muted);
  }

  .field select {
    width: 140px;
    padding-top: 2px;
    padding-bottom: 2px;
  }

  details {
    display: flex;
    flex-direction: column;
    gap: 6px;
  }

  details[open] {
    padding-bottom: 4px;
  }

  details > :global(* + *) {
    margin-top: 6px;
  }

  summary {
    cursor: pointer;
    font-size: 11px;
    font-weight: 600;
    color: var(--muted);
  }
</style>
