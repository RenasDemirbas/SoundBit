<script lang="ts">
  let {
    label,
    min,
    max,
    step = 1,
    value,
    format = String,
    title,
    onchange,
  }: {
    label: string;
    min: number;
    max: number;
    step?: number;
    value: number;
    /** Shows the committed value next to the slider. */
    format?: (value: number) => string;
    title?: string;
    /** Called once the user lets go, so every drag is a single undo step. */
    onchange: (value: number) => void;
  } = $props();
</script>

<label class="slider" {title}>
  <span>{label}</span>
  <input
    type="range"
    {min}
    {max}
    {step}
    {value}
    onchange={(event: Event & { currentTarget: HTMLInputElement }) => onchange(Number(event.currentTarget.value))}
  />
  <output>{format(value)}</output>
</label>

<style>
  .slider {
    display: grid;
    grid-template-columns: 64px 1fr 52px;
    align-items: center;
    gap: 8px;
    font-size: 11px;
    color: var(--muted);
  }

  output {
    text-align: right;
    font: 11px ui-monospace, monospace;
    color: var(--text);
    white-space: nowrap;
  }
</style>
