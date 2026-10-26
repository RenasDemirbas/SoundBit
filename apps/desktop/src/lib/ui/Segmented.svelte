<script lang="ts" generics="T extends string | number">
  let {
    options,
    value,
    label,
    onchange,
  }: {
    options: { value: T; label: string; title?: string }[];
    value: T;
    label: string;
    onchange: (value: T) => void;
  } = $props();
</script>

<div class="segmented" role="radiogroup" aria-label={label}>
  {#each options as option (option.value)}
    <button
      role="radio"
      aria-checked={option.value === value}
      class:selected={option.value === value}
      title={option.title ?? option.label}
      onclick={() => onchange(option.value)}>{option.label}</button
    >
  {/each}
</div>

<style>
  .segmented {
    display: inline-flex;
    padding: 2px;
    gap: 2px;
    border-radius: 7px;
    background: var(--bg);
    border: 1px solid var(--border);
  }

  button {
    padding: 2px 8px;
    border: none;
    border-radius: 5px;
    background: transparent;
    color: var(--muted);
    font-size: 12px;
  }

  button:hover {
    color: var(--text);
  }

  button.selected {
    background: var(--panel-raised);
    color: var(--accent);
  }
</style>
