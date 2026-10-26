import { describe, expect, it } from 'vitest';

import { cellAt, gridSize, instrumentColor, type Grid } from './grid';

const grid: Grid = { labelWidth: 40, headerHeight: 20, cellWidth: 10, cellHeight: 5, columns: 4, rows: 3 };

describe('grid geometry', () => {
  it('finds the cell under the pointer', () => {
    expect(cellAt(grid, 40, 20)).toEqual({ column: 0, row: 0 });
    expect(cellAt(grid, 79, 34)).toEqual({ column: 3, row: 2 });
  });

  it('ignores labels, header and the outside', () => {
    expect(cellAt(grid, 39, 30)).toBeNull();
    expect(cellAt(grid, 50, 19)).toBeNull();
    expect(cellAt(grid, 80, 30)).toBeNull();
    expect(cellAt(grid, 50, 35)).toBeNull();
  });

  it('measures the canvas', () => {
    expect(gridSize(grid)).toEqual({ width: 80, height: 35 });
  });

  it('cycles instrument colors', () => {
    const palette = ['#000001', '#000002', '#000003'];
    expect(instrumentColor(0, palette)).toBe('#000001');
    expect(instrumentColor(4, palette)).toBe('#000002');
    expect(instrumentColor(-1, palette)).toBe('#000003');
  });
});
