<script lang="ts">
  import { onMount } from 'svelte';

  import ArrangementView from './lib/components/ArrangementView.svelte';
  import InstrumentPanel from './lib/components/InstrumentPanel.svelte';
  import PatternEditor from './lib/components/PatternEditor.svelte';
  import PatternPanel from './lib/components/PatternPanel.svelte';
  import StatusBar from './lib/components/StatusBar.svelte';
  import TopBar from './lib/components/TopBar.svelte';
  import { editor } from './lib/stores/editor.svelte';
  import { transport } from './lib/stores/transport.svelte';
  import { theme } from './lib/theme/theme.svelte';

  onMount(() => {
    theme.init();
    void editor.init();
  });

  /** Global shortcuts, ignored while typing in a field. */
  function onKeydown(event: KeyboardEvent): void {
    const target = event.target;
    if (target instanceof HTMLInputElement || target instanceof HTMLSelectElement) return;
    const command = event.ctrlKey || event.metaKey;
    const key = event.key.toLowerCase();
    if (event.key === ' ') {
      event.preventDefault();
      void transport.toggle();
    } else if (command && key === 'z') {
      event.preventDefault();
      void (event.shiftKey ? editor.redo() : editor.undo());
    } else if (command && key === 'y') {
      event.preventDefault();
      void editor.redo();
    } else if (command && key === 's') {
      event.preventDefault();
      void editor.save(event.shiftKey);
    } else if (command && key === 'o') {
      event.preventDefault();
      void editor.open();
    } else if (command && key === 'n') {
      event.preventDefault();
      void editor.newSong();
    } else if (command && key === 'e') {
      event.preventDefault();
      void editor.exportWav();
    }
  }
</script>

<svelte:window onkeydown={onKeydown} />

<div class="app">
  <TopBar />
  {#if editor.song}
    <main class="workspace">
      <aside class="sidebar">
        <InstrumentPanel />
        <PatternPanel />
      </aside>
      <section class="center">
        <ArrangementView />
        <PatternEditor />
      </section>
    </main>
  {:else}
    <main class="loading muted">Loading…</main>
  {/if}
  <StatusBar />
</div>

<style>
  .app {
    display: grid;
    grid-template-rows: auto 1fr auto;
    height: 100%;
  }

  .workspace {
    display: grid;
    grid-template-columns: 260px 1fr;
    gap: 8px;
    padding: 8px;
    min-height: 0;
  }

  /* The pattern panel takes the remaining height and scrolls its own list. */
  .sidebar {
    display: flex;
    flex-direction: column;
    gap: 8px;
    min-height: 0;
    overflow: hidden;
  }

  .center {
    display: grid;
    grid-template-rows: minmax(240px, 52%) 1fr;
    gap: 8px;
    min-height: 0;
    min-width: 0;
  }

  .loading {
    display: grid;
    place-items: center;
  }
</style>
