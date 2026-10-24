import { it, expect } from 'vitest';
import { storageColor, treemap } from './treemap';
it('preserves relative area without overlaps or out-of-bounds tiles', () => {
  const weights = [45, 22, 13, 8, 7, 5, 0];
  const result = treemap(weights);
  for (let i = 0; i < result.length; i++) {
    const r = result[i];
    expect(r.left).toBeGreaterThanOrEqual(0);
    expect(r.top).toBeGreaterThanOrEqual(0);
    expect(r.left + r.width).toBeLessThanOrEqual(100.0001);
    expect(r.top + r.height).toBeLessThanOrEqual(100.0001);
    expect(r.width * r.height).toBeCloseTo(weights[i] * 100, 5);
    for (let j = i + 1; j < result.length; j++) {
      const s = result[j];
      const overlap =
        Math.max(
          0,
          Math.min(r.left + r.width, s.left + s.width) -
            Math.max(r.left, s.left),
        ) *
        Math.max(
          0,
          Math.min(r.top + r.height, s.top + s.height) - Math.max(r.top, s.top),
        );
      expect(overlap).toBeCloseTo(0, 5);
    }
  }
});

it('keeps bounded geometry for skewed small-file groups, zero and invalid sizes', () => {
  const weights = [
    9_000_000_000,
    ...Array.from({ length: 60 }, () => 4096),
    0,
    -9,
    NaN,
    Infinity,
  ];
  const result = treemap(weights, 1.25);
  const total = 9_000_000_000 + 60 * 4096;
  for (let i = 0; i < result.length; i++) {
    const r = result[i];
    expect(Object.values(r).every(Number.isFinite)).toBe(true);
    expect(r.left + r.width).toBeLessThanOrEqual(100.00001);
    expect(r.top + r.height).toBeLessThanOrEqual(100.00001);
    expect(r.width * r.height).toBeCloseTo(
      i <= 60 ? (weights[i] / total) * 10_000 : 0,
      5,
    );
  }
  expect(treemap([0, 0])).toEqual([
    { left: 0, top: 0, width: 0, height: 0 },
    { left: 0, top: 0, width: 0, height: 0 },
  ]);
});

it('uses deterministic colors for the same path across map and table', () => {
  expect(storageColor('/fixture/target/debug')).toBe(
    storageColor('/fixture/target/debug'),
  );
  expect(storageColor('/fixture/target/debug')).toBeGreaterThanOrEqual(0);
  expect(storageColor('/fixture/target/debug')).toBeLessThan(7);
});
