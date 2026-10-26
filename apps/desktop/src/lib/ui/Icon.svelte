<script lang="ts" module>
  export type IconName =
    | 'play'
    | 'stop'
    | 'loop'
    | 'undo'
    | 'redo'
    | 'chevron'
    | 'sliders'
    | 'plus'
    | 'trash'
    | 'speaker'
    | 'mute'
    | 'copy'
    | 'search'
    | 'help'
    | 'palette';
</script>

<script lang="ts">
  let { name, size = 16 }: { name: IconName; size?: number } = $props();

  /** 24×24 paths. Filled icons use `fill`, the rest are drawn with strokes. */
  const PATHS: Record<IconName, string> = {
    play: 'M8 5.5v13l10.5-6.5z',
    stop: 'M7 7h10v10H7z',
    loop: 'M17 2l4 4-4 4M3 11V10a4 4 0 0 1 4-4h14M7 22l-4-4 4-4M21 13v1a4 4 0 0 1-4 4H3',
    undo: 'M9 14L4 9l5-5M4 9h10.5a5.5 5.5 0 0 1 0 11H11',
    redo: 'M15 14l5-5-5-5M20 9H9.5a5.5 5.5 0 0 0 0 11H13',
    chevron: 'M6 9l6 6 6-6',
    sliders: 'M4 21v-7M4 10V3M12 21v-9M12 8V3M20 21v-5M20 12V3M1 14h6M9 8h6M17 16h6',
    plus: 'M12 5v14M5 12h14',
    trash: 'M4 7h16M10 7V4h4v3M6 7l1 13h10l1-13M10 11v6M14 11v6',
    speaker: 'M4 9v6h4l5 4V5L8 9zM16.5 8.5a5 5 0 0 1 0 7',
    mute: 'M4 9v6h4l5 4V5L8 9zM16 9l5 6M21 9l-5 6',
    copy: 'M9 9h11v11H9zM5 15V4h11',
    search: 'M11 4a7 7 0 1 0 0 14a7 7 0 1 0 0-14M20 20l-4.2-4.2',
    palette:
      'M12 3a9 9 0 1 0 0 18c1.1 0 1.6-.8 1.6-1.6 0-.9-.8-1.3-.8-2.2 0-.9.7-1.6 1.6-1.6H17a4 4 0 0 0 4-4c0-4.7-4-8.6-9-8.6M7.5 11.5h.01M9.5 7.5h.01M14.5 7.5h.01M16.5 11h.01',
    help: 'M12 2.5a9.5 9.5 0 1 0 0 19a9.5 9.5 0 1 0 0-19M9.6 9.2a2.5 2.5 0 1 1 3.4 2.4c-.6.3-1 .8-1 1.5v.4M12 17h.01',
  };
  const FILLED: IconName[] = ['play', 'stop'];
  const filled = $derived(FILLED.includes(name));
</script>

<svg
  width={size}
  height={size}
  viewBox="0 0 24 24"
  fill={filled ? 'currentColor' : 'none'}
  stroke={filled ? 'none' : 'currentColor'}
  stroke-width="2"
  stroke-linecap="round"
  stroke-linejoin="round"
  aria-hidden="true"
>
  <path d={PATHS[name]} />
</svg>
