import { it, expect } from 'vitest';
import { treemap } from './treemap';
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
