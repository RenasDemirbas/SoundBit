<script lang="ts">
  import { profileLabel, scaleLabel } from '../music/options';
  import { className } from '../music/pitch';
  import { editor } from '../stores/editor.svelte';
  import { theme } from '../theme/theme.svelte';
  import { THEMES } from '../theme/themes';
  import BrandMark from '../ui/BrandMark.svelte';
  import Icon from '../ui/Icon.svelte';
  import Menu, { type MenuEntry } from '../ui/Menu.svelte';
  import Popover from '../ui/Popover.svelte';
  import SongSettings from './SongSettings.svelte';
  import TransportControls from './TransportControls.svelte';

  const song = $derived(editor.song);
  const fileMenu: MenuEntry[] = [
    { label: 'New song', shortcut: 'Mod+N', action: () => void editor.newSong() },
    { label: 'Open…', shortcut: 'Mod+O', action: () => void editor.open() },
    null,
    { label: 'Save', shortcut: 'Mod+S', action: () => void editor.save() },
    { label: 'Save as…', shortcut: 'Mod+Shift+S', action: () => void editor.save(true) },
    null,
    { label: 'Export WAV…', shortcut: 'Mod+E', action: () => void editor.exportWav() },
  ];

  const themeMenu: MenuEntry[] = $derived(
    THEMES.map((option) => ({
      label: option.name,
      checked: option.id === theme.current.id,
      swatch: option.ui['accent'],
      action: () => theme.select(option.id),
    })),
  );

  function setTitle(event: Event & { currentTarget: HTMLInputElement }): void {
    const title = event.currentTarget.value.trim();
    if (title && title !== song?.meta.title) void editor.act({ kind: 'update_song', title });
    else event.currentTarget.value = song?.meta.title ?? '';
  }

  const blurOnEnter = (event: KeyboardEvent & { currentTarget: HTMLInputElement }): void => {
    if (event.key === 'Enter') event.currentTarget.blur();
  };
</script>

<header class="top-bar">
  <div class="side">
    <span class="logo" title="SoundBit"><BrandMark size={30} /></span>
    <Menu label="File" items={fileMenu} />
    <div class="title">
      <input
        value={song?.meta.title ?? ''}
        onchange={setTitle}
        onkeydown={blurOnEnter}
        aria-label="Song title"
        spellcheck="false"
      />
      {#if editor.state?.modified}<span class="dot" title="Unsaved changes"></span>{/if}
    </div>
  </div>

  <TransportControls />

  <div class="side end">
    {#if song}
      <Popover title="Song settings" align="right">
        {#snippet trigger()}
          <Icon name="sliders" size={14} />
          <span class="summary">
            {className(song.scale.root)} {scaleLabel(song.scale.kind).toLowerCase()}
            <span class="muted">·</span>
            {profileLabel(song.chip_profile)}
          </span>
          <Icon name="chevron" size={12} />
        {/snippet}
        <SongSettings {song} />
      </Popover>
    {/if}
    <Menu label="Color theme" icon="palette" items={themeMenu} align="right" />
    <span class="divider"></span>
    <button
      class="ghost icon"
      disabled={!editor.state?.undo}
      title={editor.state?.undo ? `Undo: ${editor.state.undo}` : 'Nothing to undo'}
      aria-label="Undo"
      onclick={() => void editor.undo()}
    >
      <Icon name="undo" />
    </button>
    <button
      class="ghost icon"
      disabled={!editor.state?.redo}
      title={editor.state?.redo ? `Redo: ${editor.state.redo}` : 'Nothing to redo'}
      aria-label="Redo"
      onclick={() => void editor.redo()}
    >
      <Icon name="redo" />
    </button>
  </div>
</header>

<style>
  .top-bar {
    display: grid;
    grid-template-columns: 1fr auto 1fr;
    align-items: center;
    gap: 12px;
    height: 52px;
    padding: 0 12px;
    background: var(--panel);
    border-bottom: 1px solid var(--border);
  }

  .side {
    display: flex;
    align-items: center;
    gap: 4px;
    min-width: 0;
  }

  .end {
    justify-content: flex-end;
  }

  .logo {
    display: grid;
    margin-right: 4px;
  }

  .title {
    display: flex;
    align-items: center;
    gap: 6px;
    min-width: 0;
    margin-left: 6px;
  }

  .title input {
    min-width: 0;
    width: 240px;
    padding: 4px 6px;
    border-color: transparent;
    background: transparent;
    font-size: 14px;
    font-weight: 600;
  }

  .title input:hover {
    border-color: var(--border);
  }

  .title input:focus {
    border-color: var(--accent);
    background: var(--bg);
  }

  .dot {
    width: 7px;
    height: 7px;
    border-radius: 50%;
    background: var(--accent-2);
    flex: none;
  }

  .icon {
    display: grid;
    place-items: center;
    width: 32px;
    height: 32px;
    padding: 0;
  }

  /* One line, shortened with an ellipsis; only the icon remains on narrow windows. */
  .summary {
    max-width: 220px;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  @media (max-width: 1100px) {
    .summary {
      display: none;
    }

    .title input {
      width: 160px;
    }
  }

  .divider {
    width: 1px;
    height: 22px;
    margin: 0 6px;
    background: var(--border);
  }
</style>
