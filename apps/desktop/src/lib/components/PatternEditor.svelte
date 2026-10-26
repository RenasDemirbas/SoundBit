<script lang="ts">
  import { theme } from '../theme/theme.svelte';
  import { cellAt, prepareCanvas } from '../canvas/grid';
  import { drawPianoRoll, pitchAtRow, rollGrid, rowOfPitch } from '../canvas/pianoRoll';
  import { editor } from '../stores/editor.svelte';
  import { transport } from '../stores/transport.svelte';
  import Segmented from '../ui/Segmented.svelte';

  const LENGTHS = [1, 2, 4, 8, 16].map((steps) => ({
    value: steps,
    label: String(steps),
    title: `New notes last ${steps} step${steps === 1 ? '' : 's'}`,
  }));
  /** Pitch centred in view for an empty pattern. */
  const DEFAULT_FOCUS_PITCH = 64;

  let canvas = $state<HTMLCanvasElement>();
  let scroller = $state<HTMLDivElement>();
  let width = $state(0);
  const editorState = $derived(editor.state);
  const pattern = $derived(editor.pattern);
  const grid = $derived(rollGrid(editorState?.steps_per_bar ?? 16, width));
  const instrumentIndex = $derived(editorState?.song.instruments.findIndex((i) => i.id === pattern?.instrument) ?? 0);

  /** The step under the play head, if this pattern is playing on some track right now. */
  const playStep = $derived.by(() => {
    const position = transport.position;
    if (position === null || !pattern || !editorState) return null;
    const bar = Math.floor(position);
    const playing = editorState.song.tracks.some((t) => t.clips.some((c) => c.bar === bar && c.pattern === pattern.id));
    return playing ? Math.floor((position - bar) * editorState.steps_per_bar) : null;
  });

  $effect(() => {
    const context = canvas && editorState ? prepareCanvas(canvas, grid) : null;
    if (!context || !editorState) return;
    drawPianoRoll(context, {
      grid,
      notes: pattern?.notes ?? [],
      stepsPerBeat: editorState.song.timing.steps_per_beat,
      scaleClasses: editorState.scale_classes,
      color: theme.instrumentColor(instrumentIndex),
      playStep,
      colors: theme.current.canvas,
    });
  });

  /** When another pattern is selected, centre its notes in view. */
  let focusedPattern: number | null = null;
  $effect(() => {
    if (!scroller || !pattern || pattern.id === focusedPattern) return;
    focusedPattern = pattern.id;
    const pitches = pattern.notes.map((note) => note.pitch);
    const middle = pitches.length ? (Math.max(...pitches) + Math.min(...pitches)) / 2 : DEFAULT_FOCUS_PITCH;
    scroller.scrollTop = (rowOfPitch(Math.round(middle)) + 0.5) * grid.cellHeight - scroller.clientHeight / 2;
  });

  const steps = $derived(Array.from({ length: grid.columns }, (_, step) => step));

  function onClick(event: MouseEvent): void {
    const cell = cellAt(grid, event.offsetX, event.offsetY);
    if (!cell || !pattern) return;
    const pitch = pitchAtRow(cell.row);
    void editor.act({ kind: 'toggle_note', pattern: pattern.id, start: cell.column, pitch, length: editor.noteLength });
  }

  function rename(event: Event & { currentTarget: HTMLInputElement }): void {
    const name = event.currentTarget.value.trim();
    if (pattern && name && name !== pattern.name) void editor.act({ kind: 'update_pattern', id: pattern.id, name });
  }

  function setInstrument(event: Event & { currentTarget: HTMLSelectElement }): void {
    if (pattern) {
      void editor.act({ kind: 'update_pattern', id: pattern.id, instrument: Number(event.currentTarget.value) });
    }
  }

  function transpose(semitones: number): void {
    if (pattern) void editor.act({ kind: 'transpose_pattern', id: pattern.id, semitones });
  }
</script>

<section class="panel roll">
  {#if pattern}
    <div class="toolbar">
      <span class="swatch" style:background={theme.instrumentColor(instrumentIndex)}></span>
      <input class="pattern-name" value={pattern.name} onchange={rename} aria-label="Pattern name" spellcheck="false" />
      <select class="instrument" value={String(pattern.instrument)} onchange={setInstrument} aria-label="Instrument">
        {#each editorState?.song.instruments ?? [] as instrument (instrument.id)}
          <option value={String(instrument.id)}>{instrument.name}</option>
        {/each}
      </select>
      <span class="spacer"></span>
      <span class="group-label">Note</span>
      <Segmented label="Note length" options={LENGTHS} value={editor.noteLength} onchange={(length) => (editor.noteLength = length)} />
      <span class="group-label">Transpose</span>
      <div class="button-group" role="group" aria-label="Transpose">
        <button onclick={() => transpose(-12)} title="Down an octave">−8va</button>
        <button onclick={() => transpose(-1)} title="Down a semitone">−1</button>
        <button onclick={() => transpose(1)} title="Up a semitone">+1</button>
        <button onclick={() => transpose(12)} title="Up an octave">+8va</button>
      </div>
    </div>
    <div class="steps" style:padding-left={`${grid.labelWidth}px`}>
      {#each steps as step (step)}
        <span
          style:width={`${grid.cellWidth}px`}
          class:beat={step % (editorState?.song.timing.steps_per_beat ?? 4) === 0}>{step}</span
        >
      {/each}
    </div>
    <div class="scroll" bind:this={scroller} bind:clientWidth={width}>
      <canvas bind:this={canvas} onclick={onClick}></canvas>
    </div>
  {:else}
    <p class="muted empty">Create or select a pattern to edit its notes.</p>
  {/if}
</section>

<style>
  .roll {
    display: flex;
    flex-direction: column;
    gap: 8px;
    min-height: 0;
  }

  .toolbar {
    display: flex;
    align-items: center;
    gap: 8px;
    flex-wrap: wrap;
  }

  .pattern-name {
    width: 200px;
    border-color: transparent;
    background: transparent;
    font-size: 14px;
    font-weight: 600;
  }

  .pattern-name:hover {
    border-color: var(--border);
  }

  .pattern-name:focus {
    border-color: var(--accent);
    background: var(--bg);
  }

  .instrument {
    padding-top: 2px;
    padding-bottom: 2px;
  }

  .spacer {
    flex: 1;
  }

  .group-label {
    font-size: 10px;
    letter-spacing: 0.08em;
    text-transform: uppercase;
    color: var(--muted);
  }

  .steps {
    display: flex;
    font: 10px ui-monospace, monospace;
    color: var(--muted);
  }

  .steps span {
    flex: none;
    padding-left: 4px;
  }

  .steps .beat {
    color: var(--text);
  }

  .scroll {
    overflow: auto;
    min-height: 0;
    flex: 1;
  }

  canvas {
    display: block;
    cursor: crosshair;
  }

  .empty {
    margin: auto;
  }
</style>
