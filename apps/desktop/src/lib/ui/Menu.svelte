<script lang="ts" module>
  export interface MenuItem {
    label: string;
    /** Shortcut in `shortcutLabel` notation, e.g. "Mod+S". */
    shortcut?: string;
    disabled?: boolean;
    /** Shows a check mark, for the active choice in a list. */
    checked?: boolean;
    /** A small color sample in front of the label. */
    swatch?: string;
    /** Tooltip with more detail. */
    title?: string;
    action: () => void;
  }

  /** A menu entry, or `null` for a separator line. */
  export type MenuEntry = MenuItem | null;
</script>

<script lang="ts">
  import { clickOutside } from './clickOutside';
  import Icon, { type IconName } from './Icon.svelte';
  import { shortcutLabel } from './shortcuts';

  let {
    label,
    items,
    icon,
    align = 'left',
  }: {
    label: string;
    items: MenuEntry[];
    /** Show only this icon as the trigger; `label` becomes its tooltip. */
    icon?: IconName;
    align?: 'left' | 'right';
  } = $props();
  let open = $state(false);

  function choose(item: MenuItem): void {
    open = false;
    item.action();
  }
</script>

<div class="menu" use:clickOutside={() => (open = false)}>
  <button
    class={icon ? 'icon-button' : 'ghost trigger'}
    class:open
    title={icon ? label : undefined}
    aria-label={label}
    aria-haspopup="menu"
    aria-expanded={open}
    onclick={() => (open = !open)}
  >
    {#if icon}
      <Icon name={icon} />
    {:else}
      {label}
      <Icon name="chevron" size={12} />
    {/if}
  </button>
  {#if open}
    <div class="dropdown" class:right={align === 'right'} role="menu">
      {#each items as item, index (index)}
        {#if item}
          <button role="menuitem" disabled={item.disabled} title={item.title} onclick={() => choose(item)}>
            <span class="label">
              {#if item.checked !== undefined}<span class="check">{item.checked ? '✓' : ''}</span>{/if}
              {#if item.swatch}<span class="swatch" style:background={item.swatch}></span>{/if}
              {item.label}
            </span>
            {#if item.shortcut}<kbd>{shortcutLabel(item.shortcut)}</kbd>{/if}
          </button>
        {:else}
          <hr />
        {/if}
      {/each}
    </div>
  {/if}
</div>

<style>
  .menu {
    position: relative;
  }

  .trigger {
    display: flex;
    align-items: center;
    gap: 4px;
  }

  .dropdown {
    position: absolute;
    top: calc(100% + 6px);
    left: 0;
    z-index: 20;
    min-width: 220px;
    padding: 4px;
    background: var(--panel-raised);
    border: 1px solid var(--border);
    border-radius: 8px;
    box-shadow: var(--shadow);
    max-height: 70vh;
    overflow-y: auto;
  }

  .dropdown.right {
    left: auto;
    right: 0;
  }

  [role='menuitem'] {
    display: flex;
    justify-content: space-between;
    gap: 24px;
    width: 100%;
    border: none;
    background: transparent;
    padding: 6px 10px;
    text-align: left;
  }

  [role='menuitem']:hover:not(:disabled) {
    background: var(--border);
  }

  .label {
    display: flex;
    align-items: center;
    gap: 8px;
  }

  .check {
    width: 10px;
    color: var(--accent);
  }

  kbd {
    font: inherit;
    font-size: 11px;
    color: var(--muted);
  }

  hr {
    border: none;
    border-top: 1px solid var(--border);
    margin: 4px 2px;
  }
</style>
