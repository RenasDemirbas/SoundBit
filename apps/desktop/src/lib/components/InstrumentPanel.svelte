<script lang="ts">
  import { theme } from '../theme/theme.svelte';
  import type { Voice } from '../ipc/types';
  import { categoryLabel, groupPresets } from '../music/presets';
  import { editor } from '../stores/editor.svelte';
  import Menu, { type MenuEntry } from '../ui/Menu.svelte';
  import PanelHeader from '../ui/PanelHeader.svelte';
  import InstrumentEditor from './InstrumentEditor.svelte';

  const instruments = $derived(editor.song?.instruments ?? []);
  /** The preset library, one block per category with the category as a disabled heading. */
  const presets: MenuEntry[] = $derived(
    groupPresets(editor.presets).flatMap(({ category, presets: members }, index) => [
      ...(index > 0 ? [null] : []),
      { label: categoryLabel(category), disabled: true, action: () => {} },
      ...members.map((preset) => ({
        label: preset.name,
        title: preset.description,
        action: () => void editor.act({ kind: 'add_preset', preset: preset.id }),
      })),
    ]),
  );

  const DUTY_PERCENT = { eighth: '12.5', quarter: '25', half: '50', three_quarters: '75' } as const;

  /** A short description of the voice, e.g. "pulse 25%". */
  function voiceLabel(voice: Voice): string {
    if (voice.type === 'pulse') return `pulse ${DUTY_PERCENT[voice.duty]}%`;
    if (voice.type === 'noise') return voice.mode === 'short' ? 'noise metal' : 'noise';
    return voice.type;
  }
</script>

<section class="panel">
  <PanelHeader title="Instruments" count={instruments.length}>
    <Menu label="Add instrument" icon="plus" items={presets} align="right" />
  </PanelHeader>
  <ul class="list instruments">
    {#each instruments as instrument, index (instrument.id)}
      <li class:selected={instrument.id === editor.selectedInstrument}>
        <button onclick={() => (editor.selectedInstrument = instrument.id)}>
          <span class="swatch" style:background={theme.instrumentColor(index)}></span>
          <span class="name">{instrument.name}</span>
          <span class="badge">{voiceLabel(instrument.voice)}</span>
        </button>
      </li>
    {:else}
      <li class="muted empty">Add an instrument with <strong>+</strong>.</li>
    {/each}
  </ul>
  {#if editor.instrument}
    <InstrumentEditor instrument={editor.instrument} />
  {/if}
</section>

<style>
  .instruments {
    max-height: 176px;
    overflow-y: auto;
    margin-top: 4px;
  }

  .name {
    flex: 1;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .badge {
    font: 10px ui-monospace, monospace;
    color: var(--muted);
  }

  .empty {
    padding: 6px 2px;
  }
</style>
