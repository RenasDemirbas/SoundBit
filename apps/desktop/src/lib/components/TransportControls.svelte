<script lang="ts">
  import { formatTime } from '../music/time';
  import { editor } from '../stores/editor.svelte';
  import { transport } from '../stores/transport.svelte';
  import Icon from '../ui/Icon.svelte';

  const song = $derived(editor.song);

  /** Bar and beat under the play head (bars counted from 0, like the arrangement). */
  const position = $derived.by(() => {
    const bars = transport.position ?? 0;
    const bar = Math.floor(bars);
    const beat = Math.floor((bars - bar) * (song?.timing.beats_per_bar ?? 4)) + 1;
    return { bar, beat };
  });

  const elapsed = $derived(formatTime(transport.seconds ?? 0, true));
  const total = $derived(formatTime(editor.state?.length_seconds ?? 0));

  function setBpm(event: Event & { currentTarget: HTMLInputElement }): void {
    const bpm = Number(event.currentTarget.value);
    if (Number.isInteger(bpm) && bpm !== song?.timing.bpm) void editor.act({ kind: 'update_song', bpm });
  }
</script>

<div class="transport">
  <button
    class="play"
    class:playing={transport.playing}
    onclick={() => void transport.toggle()}
    title={transport.playing ? 'Stop (Space)' : 'Play (Space)'}
    aria-label={transport.playing ? 'Stop' : 'Play'}
  >
    <Icon name={transport.playing ? 'stop' : 'play'} size={18} />
  </button>
  <button
    class="ghost icon"
    class:on={transport.looping}
    onclick={() => (transport.looping = !transport.looping)}
    title={transport.looping ? 'Loop on' : 'Loop off'}
    aria-pressed={transport.looping}
  >
    <Icon name="loop" />
  </button>

  <div class="readout" title="Position">
    <span class="value">{String(position.bar).padStart(2, '0')}<span class="dim">.{position.beat}</span></span>
    <span class="unit">bar</span>
  </div>
  <div class="readout time" title="Elapsed / song length">
    <span class="value">{elapsed}</span>
    <span class="dim">/ {total}</span>
  </div>
  <label class="readout" title="Tempo">
    <input class="value" type="number" min="20" max="400" value={song?.timing.bpm ?? 120} onchange={setBpm} />
    <span class="unit">bpm</span>
  </label>
</div>

<style>
  .transport {
    display: flex;
    align-items: center;
    gap: 6px;
    padding: 3px 10px 3px 3px;
    background: var(--bg);
    border: 1px solid var(--border);
    border-radius: 999px;
  }

  .play {
    display: grid;
    place-items: center;
    width: 32px;
    height: 32px;
    padding: 0;
    border: none;
    border-radius: 50%;
    background: var(--accent);
    color: var(--bg);
  }

  .play.playing {
    background: var(--accent-2);
  }

  .icon {
    display: grid;
    place-items: center;
    width: 30px;
    height: 30px;
    padding: 0;
    color: var(--muted);
  }

  .icon.on {
    color: var(--accent);
  }

  .readout {
    display: flex;
    align-items: baseline;
    gap: 4px;
    padding: 0 6px;
    font-family: ui-monospace, monospace;
  }

  .value {
    font-size: 15px;
    color: var(--text);
  }

  input.value {
    width: 46px;
    padding: 0;
    border: none;
    background: transparent;
    font-family: inherit;
    text-align: right;
    appearance: textfield;
  }

  input.value::-webkit-inner-spin-button {
    appearance: none;
  }

  .time {
    min-width: 108px;
    justify-content: center;
    border-left: 1px solid var(--border);
    border-right: 1px solid var(--border);
  }

  .time .dim {
    font-size: 12px;
  }

  .dim,
  .unit {
    color: var(--muted);
  }

  .unit {
    font-size: 10px;
    text-transform: uppercase;
    letter-spacing: 0.06em;
  }
</style>
