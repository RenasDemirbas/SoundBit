<script lang="ts">
  import type { Snippet } from 'svelte';

  let { title, count, children }: { title: string; count?: number; children?: Snippet } = $props();
</script>

<header class="panel-header">
  <h2>{title}{#if count !== undefined}<span class="count">{count}</span>{/if}</h2>
  {#if children}<div class="actions">{@render children()}</div>{/if}
</header>

<style>
  .panel-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 8px;
    min-height: 30px;
  }

  h2 {
    display: flex;
    align-items: center;
    gap: 6px;
    margin: 0;
  }

  .count {
    padding: 0 6px;
    border-radius: 999px;
    background: var(--border);
    color: var(--muted);
    font-size: 10px;
    letter-spacing: 0;
  }

  .actions {
    display: flex;
    align-items: center;
    gap: 2px;
  }
</style>
