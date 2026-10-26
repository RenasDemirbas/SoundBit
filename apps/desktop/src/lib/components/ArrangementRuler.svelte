<script lang="ts">
  let {
    bars,
    barWidth,
    loop,
    position,
    onplay,
  }: {
    bars: number;
    barWidth: number;
    loop: { start: number; end: number } | null;
    position: number | null;
    onplay: (bar: number) => void;
  } = $props();

  const numbers = $derived(Array.from({ length: bars }, (_, bar) => bar));

  function onClick(event: MouseEvent & { currentTarget: HTMLDivElement }): void {
    const left = event.currentTarget.getBoundingClientRect().left;
    onplay(Math.max(0, Math.floor((event.clientX - left) / barWidth)));
  }
</script>

<!-- Clicking a bar number starts playback there; the keyboard equivalent is Space. -->
<!-- svelte-ignore a11y_click_events_have_key_events -->
<div class="ruler" role="button" tabindex="-1" title="Click to play from this bar" onclick={onClick}
  style:width={`${bars * barWidth}px`}>
  {#if loop}
    <div class="loop" style:left={`${loop.start * barWidth}px`} style:width={`${(loop.end - loop.start) * barWidth}px`}></div>
  {/if}
  {#each numbers as bar (bar)}
    <span class:major={bar % 4 === 0} style:width={`${barWidth}px`}>{bar}</span>
  {/each}
  {#if position !== null}
    <div class="head" style:left={`${position * barWidth}px`}></div>
  {/if}
</div>

<style>
  .ruler {
    position: relative;
    display: flex;
    height: 100%;
    cursor: pointer;
    font: 10px ui-monospace, monospace;
    color: var(--muted);
  }

  span {
    flex: none;
    padding: 6px 0 0 5px;
    border-left: 1px solid var(--grid-line);
  }

  span.major {
    color: var(--text);
    border-left-color: var(--grid-line-strong);
  }

  .loop {
    position: absolute;
    top: 0;
    bottom: 0;
    background: var(--loop-bg);
    border-bottom: 2px solid var(--accent-2);
  }

  .head {
    position: absolute;
    top: 0;
    bottom: 0;
    width: 0;
    border-left: 8px solid transparent;
    border-right: 8px solid transparent;
    border-top: 10px solid var(--accent);
    transform: translateX(-7px);
  }
</style>
