<script lang="ts">
  import { theme } from '../theme/theme.svelte';
  import type { Pattern } from '../ipc/types';
  import { editor } from '../stores/editor.svelte';
  import Icon from '../ui/Icon.svelte';
  import PanelHeader from '../ui/PanelHeader.svelte';

  const song = $derived(editor.song);
  let filter = $state('');
  /** Instrument ids whose pattern groups are collapsed. */
  let collapsed = $state<Record<number, boolean>>({});

  /** How many bars use each pattern. */
  const uses = $derived.by(() => {
    const counts: Record<number, number> = {};
    for (const clip of song?.tracks.flatMap((track) => track.clips) ?? []) {
      counts[clip.pattern] = (counts[clip.pattern] ?? 0) + 1;
    }
    return counts;
  });

  /** Patterns grouped by instrument, filtered by name. */
  const groups = $derived.by(() => {
    const query = filter.trim().toLowerCase();
    const matches = (pattern: Pattern): boolean => !query || pattern.name.toLowerCase().includes(query);
    return (song?.instruments ?? [])
      .map((instrument, index) => ({
        instrument,
        color: theme.instrumentColor(index),
        patterns: (song?.patterns ?? []).filter((p) => p.instrument === instrument.id && matches(p)),
      }))
      .filter((group) => group.patterns.length > 0);
  });

  function create(): void {
    const instrument = editor.selectedInstrument;
    if (instrument === null) {
      editor.notify('Add an instrument first.', true);
      return;
    }
    void editor.act({ kind: 'create_pattern', name: `Pattern ${(song?.patterns.length ?? 0) + 1}`, instrument });
  }

  function select(pattern: Pattern): void {
    editor.selectedPattern = pattern.id;
    editor.selectedInstrument = pattern.instrument;
  }
</script>

<section class="panel patterns">
  <PanelHeader title="Patterns" count={song?.patterns.length ?? 0}>
    <button class="icon-button" title="New pattern for the selected instrument" aria-label="New pattern" onclick={create}>
      <Icon name="plus" />
    </button>
  </PanelHeader>
  <label class="search">
    <Icon name="search" size={13} />
    <input placeholder="Search patterns" bind:value={filter} aria-label="Search patterns" spellcheck="false" />
  </label>
  <div class="groups">
    {#each groups as group (group.instrument.id)}
      <button
        class="group"
        style:--color={group.color}
        aria-expanded={!collapsed[group.instrument.id]}
        onclick={() => (collapsed[group.instrument.id] = !collapsed[group.instrument.id])}
      >
        <span class="arrow" class:closed={collapsed[group.instrument.id]}><Icon name="chevron" size={11} /></span>
        {group.instrument.name}
        <small>{group.patterns.length}</small>
      </button>
      {#if !collapsed[group.instrument.id]}
        <ul class="list">
          {#each group.patterns as pattern (pattern.id)}
            <li class="item" class:selected={pattern.id === editor.selectedPattern}>
              <button onclick={() => select(pattern)}>
                <span class="swatch" style:background={group.color}></span>
                <span class="name">{pattern.name}</span>
                <small class="meta">{pattern.notes.length}♪ · {uses[pattern.id] ?? 0}×</small>
              </button>
              <span class="actions">
                <button class="icon-button" title="Duplicate" aria-label="Duplicate pattern"
                  onclick={() => void editor.act({ kind: 'duplicate_pattern', id: pattern.id })}><Icon name="copy" size={13} /></button>
                <button class="icon-button" title="Delete" aria-label="Delete pattern"
                  onclick={() => void editor.act({ kind: 'remove_pattern', id: pattern.id })}><Icon name="trash" size={13} /></button>
              </span>
            </li>
          {/each}
        </ul>
      {/if}
    {:else}
      <p class="muted">{filter ? 'No pattern matches.' : 'No patterns yet.'}</p>
    {/each}
  </div>
</section>

<style>
  .patterns {
    display: flex;
    flex-direction: column;
    gap: 6px;
    min-height: 160px;
    flex: 1;
  }

  .search {
    display: flex;
    align-items: center;
    gap: 6px;
    padding: 0 8px;
    border: 1px solid var(--border);
    border-radius: 7px;
    background: var(--bg);
    color: var(--muted);
  }

  .search input {
    flex: 1;
    border: none;
    background: transparent;
    padding: 4px 0;
  }

  .groups {
    overflow-y: auto;
    flex: 1;
    min-height: 0;
  }

  .group {
    display: flex;
    gap: 6px;
    align-items: center;
    width: 100%;
    margin-top: 6px;
    padding: 3px 6px;
    border: none;
    background: transparent;
    color: var(--muted);
    font-size: 11px;
    font-weight: 600;
    letter-spacing: 0.04em;
    text-transform: uppercase;
  }

  .group:hover {
    color: var(--text);
  }

  .group small {
    margin-left: auto;
  }

  .arrow {
    display: grid;
    color: var(--color);
    transition: transform 0.1s;
  }

  .arrow.closed {
    transform: rotate(-90deg);
  }

  .item {
    position: relative;
  }

  .item .actions {
    position: absolute;
    top: 50%;
    right: 4px;
    display: none;
    transform: translateY(-50%);
    background: var(--panel-raised);
    border-radius: 6px;
  }

  .item:hover .actions {
    display: flex;
  }

  .item:hover .meta {
    visibility: hidden;
  }

  .item .actions button {
    width: 24px;
    height: 24px;
  }

  .name {
    flex: 1;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }
</style>
