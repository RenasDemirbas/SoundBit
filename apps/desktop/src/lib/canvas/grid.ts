// Geometry shared by the canvas editors: which cell is under the pointer, and the canvas size.

export interface Grid {
  /** Width of the label column on the left (piano keys, track names). */
  labelWidth: number;
  /** Height of the header row on top (step or bar numbers). */
  headerHeight: number;
  cellWidth: number;
  cellHeight: number;
  columns: number;
  rows: number;
}

export interface Cell {
  column: number;
  row: number;
}

/** The cell at canvas coordinates, or `null` over labels, header or outside the grid. */
export function cellAt(grid: Grid, x: number, y: number): Cell | null {
  const column = Math.floor((x - grid.labelWidth) / grid.cellWidth);
  const row = Math.floor((y - grid.headerHeight) / grid.cellHeight);
  if (x < grid.labelWidth || y < grid.headerHeight) return null;
  if (column >= grid.columns || row >= grid.rows) return null;
  return { column, row };
}

/** Canvas size in CSS pixels. */
export function gridSize(grid: Grid): { width: number; height: number } {
  return {
    width: grid.labelWidth + grid.columns * grid.cellWidth,
    height: grid.headerHeight + grid.rows * grid.cellHeight,
  };
}

/** Sizes a canvas for sharp drawing on high-DPI screens and returns its 2D context. */
export function prepareCanvas(canvas: HTMLCanvasElement, grid: Grid): CanvasRenderingContext2D | null {
  const { width, height } = gridSize(grid);
  const ratio = window.devicePixelRatio || 1;
  canvas.width = Math.round(width * ratio);
  canvas.height = Math.round(height * ratio);
  canvas.style.width = `${width}px`;
  canvas.style.height = `${height}px`;
  const context = canvas.getContext('2d');
  context?.setTransform(ratio, 0, 0, ratio, 0, 0);
  return context;
}

/** The color of the instrument at `index`, cycling through the theme's palette. */
export function instrumentColor(index: number, palette: readonly string[]): string {
  return palette[((index % palette.length) + palette.length) % palette.length] ?? '#ffffff';
}
