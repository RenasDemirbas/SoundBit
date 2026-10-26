<script lang="ts">
  import { theme } from '../theme/theme.svelte';
  import { BAR_WIDTH, arrangementGrid, clipLook, drawArrangement, type ClipLook } from '../canvas/arrangement';
  import { clipAction } from '../canvas/clipAction';
  import { cellAt, prepareCanvas, type Cell } from '../canvas/grid';
  import type { Track } from '../ipc/types';
  import { editor } from '../stores/editor.svelte';
  import { transport } from '../stores/transport.svelte';
  import ArrangementCorner from './ArrangementCorner.svelte';
  import ArrangementRuler from './ArrangementRuler.svelte';
  import TrackHeader from './TrackHeader.svelte';

  /** Empty bars shown after the last clip, so there is room to extend the song. */
  const SPARE_BARS = 8;

  let canvas = $state<HTMLCanvasElement>();
  let hover = $state<Cell | null>(null);
  const editorState = $derived(editor.state);
  const song = $derived(editorState?.song);
  const tracks = $derived(song?.tracks ?? []);
  const bars = $derived(Math.max((editorState?.length_bars ?? 0) + SPARE_BARS, 16));
  const grid = $derived(arrangementGrid(bars, tracks.length));

  const colorOf = (instrument: number): string =>
    theme.instrumentColor(song?.instruments.findIndex((i) => i.id === instrument) ?? 0);

  const looks = $derived.by(() => {
    // Rebuilt from scratch on every change and never mutated afterwards, so it needs no reactivity.
    // eslint-disable-next-line svelte/prefer-svelte-reactivity
    const map = new Map<number, ClipLook>();
    for (const pattern of song?.patterns ?? []) {
      map.set(pattern.id, clipLook(pattern.name, colorOf(pattern.instrument), pattern.notes));
    }
    return map;
  });

  /** A track's color: the instrument of its first clip. */
  function trackColor(track: Track): string {
    const first = track.clips[0];
    const pattern = first && song?.patterns.find((p) => p.id === first.pattern);
    return pattern ? colorOf(pattern.instrument) : 'var(--border-strong)';
  }

  $effect(() => {
    const context = canvas && editorState ? prepareCanvas(canvas, grid) : null;
    if (!context || !editorState) return;
    drawArrangement(context, {
      grid,
      tracks,
      looks,
      stepsPerBar: editorState.steps_per_bar,
      selectedPattern: editor.selectedPattern,
      hover,
      loop: editorState.song.loop_range,
      playBars: transport.position,
      colors: theme.current.canvas,
    });
  });

  function setClip(track: Track, bar: number, pattern: number | null): void {
    void editor.act({ kind: 'set_clip', track: track.id, bar, pattern });
  }

  function onClick(event: MouseEvent): void {
    const cell = cellAt(grid, event.offsetX, event.offsetY);
    const track = cell ? tracks[cell.row] : undefined;
    if (!cell || !track) return;
    const current = track.clips.find((clip) => clip.bar === cell.column)?.pattern ?? null;
    const action = clipAction(current, editor.selectedPattern, event.altKey);
    if (action.kind === 'select') editor.selectedPattern = action.pattern;
    else if (action.kind === 'place') setClip(track, cell.column, action.pattern);
    else if (action.kind === 'clear') setClip(track, cell.column, null);
  }

  function onContextMenu(event: MouseEvent): void {
    event.preventDefault();
    const cell = cellAt(grid, event.offsetX, event.offsetY);
    const track = cell ? tracks[cell.row] : undefined;
    if (cell && track) setClip(track, cell.column, null);
  }

  function onMove(event: MouseEvent): void {
    const cell = cellAt(grid, event.offsetX, event.offsetY);
    if (cell?.column !== hover?.column || cell?.row !== hover?.row) hover = cell;
  }

</script>

<section class="panel arrangement">
  <div class="scroll">
    <div class="layout" style:grid-template-columns={`var(--head-width) ${bars * BAR_WIDTH}px`}>
      <div class="corner"><ArrangementCorner trackCount={tracks.length} /></div>
      <div class="ruler">
        <ArrangementRuler
          {bars}
          barWidth={BAR_WIDTH}
          loop={song?.loop_range ?? null}
          position={transport.position}
          onplay={(bar) => void transport.play(bar)}
        />
      </div>
      <div class="heads">
        {#each tracks as track, index (track.id)}
          <TrackHeader {track} channel={editorState?.channels[index] ?? null} color={trackColor(track)} />
        {:else}
          <p class="muted empty">Add a track to start arranging.</p>
        {/each}
      </div>
      <canvas
        bind:this={canvas}
        onclick={onClick}
        oncontextmenu={onContextMenu}
        onmousemove={onMove}
        onmouseleave={() => (hover = null)}
      ></canvas>
    </div>
  </div>
</section>

<style>
  .arrangement {
    --head-width: 210px;
    display: flex;
    flex-direction: column;
    gap: 6px;
    min-height: 0;
    padding: 0;
    overflow: hidden;
  }

  .scroll {
    overflow: auto;
    flex: 1;
    min-height: 0;
  }

  .layout {
    display: grid;
    grid-template-rows: 30px auto;
    width: max-content;
  }

  /* The corner, ruler and track headers stay in view while the clips scroll. */
  .corner {
    position: sticky;
    top: 0;
    left: 0;
    z-index: 3;
    background: var(--panel);
  }

  .ruler {
    position: sticky;
    top: 0;
    z-index: 2;
    background: var(--ruler-bg);
  }

  .heads {
    position: sticky;
    left: 0;
    z-index: 2;
    background: var(--panel);
  }

  canvas {
    display: block;
    cursor: pointer;
  }

  .empty {
    padding: 10px;
    margin: 0;
  }

</style>
