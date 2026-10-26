<script lang="ts">
  import type { Snippet } from 'svelte';

  import { clickOutside } from './clickOutside';

  let {
    trigger,
    children,
    title,
    align = 'left',
  }: { trigger: Snippet; children: Snippet; title: string; align?: 'left' | 'right' } = $props();
  let open = $state(false);
</script>

<div class="popover" use:clickOutside={() => (open = false)}>
  <button class="ghost trigger" class:open {title} aria-expanded={open} onclick={() => (open = !open)}>
    {@render trigger()}
  </button>
  {#if open}
    <div class="content" class:right={align === 'right'} role="dialog" aria-label={title}>
      {@render children()}
    </div>
  {/if}
</div>

<style>
  .popover {
    position: relative;
  }

  .trigger {
    display: flex;
    align-items: center;
    gap: 6px;
  }

  .content {
    position: absolute;
    top: calc(100% + 6px);
    left: 0;
    z-index: 20;
    padding: 14px;
    background: var(--panel-raised);
    border: 1px solid var(--border);
    border-radius: 8px;
    box-shadow: var(--shadow);
  }

  .content.right {
    left: auto;
    right: 0;
  }
</style>
