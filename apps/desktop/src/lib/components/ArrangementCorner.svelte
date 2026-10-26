<script lang="ts">
  import { editor } from '../stores/editor.svelte';
  import Icon from '../ui/Icon.svelte';

  let { trackCount }: { trackCount: number } = $props();

  const HELP = [
    'Click a bar: place the selected pattern',
    'Click it again or right-click: clear',
    'Alt+click: select its pattern',
    'Click the bar numbers: play from there',
  ].join('\n');

  const addTrack = (): void => void editor.act({ kind: 'add_track', name: `Track ${trackCount + 1}` });
</script>

<div class="corner">
  <h2>Arrangement</h2>
  <span class="help" title={HELP}><Icon name="help" size={14} /></span>
  <button class="icon-button add" onclick={addTrack} title="Add track" aria-label="Add track"><Icon name="plus" /></button>
</div>

<style>
  .corner {
    display: flex;
    align-items: center;
    gap: 6px;
    height: 100%;
    padding: 0 6px 0 12px;
  }

  h2 {
    margin: 0;
  }

  .help {
    display: grid;
    color: var(--muted);
    cursor: help;
  }

  .add {
    margin-left: auto;
    width: 24px;
    height: 24px;
  }
</style>
