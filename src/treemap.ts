/** Binary area subdivision. Returned bounds are percentages for resizable HTML controls. */
export function treemap(weights: number[]) {
  const result = weights.map(() => ({ left: 0, top: 0, width: 0, height: 0 }));
  const entries = weights
    .map((weight, index) => ({ weight: Math.max(0, weight), index }))
    .filter((v) => v.weight > 0);
  function split(
    items: typeof entries,
    x: number,
    y: number,
    w: number,
    h: number,
  ) {
    if (!items.length) return;
    if (items.length === 1) {
      result[items[0].index] = {
        left: x / 10,
        top: y / 2.3,
        width: w / 10,
        height: h / 2.3,
      };
      return;
    }
    const total = items.reduce((n, v) => n + v.weight, 0);
    let sum = 0;
    let pivot = 0;
    while (pivot < items.length - 1 && sum < total / 2) {
      sum += items[pivot].weight;
      pivot++;
    }
    const ratio = sum / total;
    const first = items.slice(0, pivot);
    const second = items.slice(pivot);
    if (w >= h) {
      split(first, x, y, w * ratio, h);
      split(second, x + w * ratio, y, w * (1 - ratio), h);
    } else {
      split(first, x, y, w, h * ratio);
      split(second, x, y + h * ratio, w, h * (1 - ratio));
    }
  }
  split(entries, 0, 0, 1000, 230);
  return result;
}
