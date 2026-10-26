<script lang="ts">
  // The SoundBit symbol: an eighth note whose flag dissolves into pixels. Served from
  // `public/favicon.svg`, a copy of `assets/brand/soundbit-symbol.svg`, so the app, its window
  // icon and the repository share one artwork. Brand colours stay fixed across themes.
  let { size = 28 }: { size?: number } = $props();
</script>

<img src="/favicon.svg" width={size} height={size} alt="SoundBit" />

<style>
  img {
    display: block;
  }
</style>
