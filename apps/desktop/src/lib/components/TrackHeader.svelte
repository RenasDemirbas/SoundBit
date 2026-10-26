<script lang="ts">
  import { TRACK_HEIGHT } from '../canvas/arrangement';
  import type { Track } from '../ipc/types';
  import { editor } from '../stores/editor.svelte';
  import Icon from '../ui/Icon.svelte';

  let { track, channel, color }: { track: Track; channel: string | null; color: string } = $props();

  function rename(event: Event & { currentTarget: HTMLInputElement }): void {
    const name = event.currentTarget.value.trim();
    if (name && name !== track.name) void editor.act({ kind: 'update_track', id: track.id, name });
  }
</script>

<div class="head" class:muted={track.muted} style:height={`${TRACK_HEIGHT}px`} style:--color={color}>
  <div class="label">
    <input value={track.name} onchange={rename} aria-label="Track name" spellcheck="false" />
    <small>{channel ?? `${track.clips.length} clip${track.clips.length === 1 ? '' : 's'}`}</small>
  </div>
  <button
    class="icon-button remove"
    title="Remove track"
    aria-label="Remove track"
    onclick={() => void editor.act({ kind: 'remove_track', id: track.id })}><Icon name="trash" size={14} /></button
  >
  <button
    class="icon-button"
    class:on={!track.muted}
    title={track.muted ? 'Unmute' : 'Mute'}
    aria-label={track.muted ? 'Unmute track' : 'Mute track'}
    aria-pressed={track.muted}
    onclick={() => void editor.act({ kind: 'update_track', id: track.id, muted: !track.muted })}
  >
    <Icon name={track.muted ? 'mute' : 'speaker'} size={15} />
  </button>
</div>

<style>
  .head {
    display: flex;
    align-items: center;
    gap: 2px;
    padding-right: 6px;
    border-left: 3px solid var(--color);
    border-bottom: 1px solid var(--grid-line);
    background: var(--panel);
  }

  .head.muted {
    border-left-color: var(--border);
  }

  .head.muted .label {
    opacity: 0.5;
  }

  .label {
    display: flex;
    flex-direction: column;
    flex: 1;
    min-width: 0;
    padding-left: 6px;
  }

  input {
    border-color: transparent;
    background: transparent;
    padding: 1px 4px;
    font-weight: 600;
  }

  input:focus {
    border-color: var(--accent);
    background: var(--bg);
  }

  small {
    padding-left: 5px;
    font-size: 10px;
    white-space: nowrap;
    overflow: hidden;
  }

  .remove {
    visibility: hidden;
  }

  .head:hover .remove {
    visibility: visible;
  }
</style>
