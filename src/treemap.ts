/** Squarified areas in percentages, retaining source order for accessible rows. */
export function treemap(weights: number[], aspect = 1.5) {
  const result = weights.map(() => ({ left: 0, top: 0, width: 0, height: 0 }));
  const entries = weights
    .map((weight, index) => ({ weight, index }))
    .filter((v) => Number.isFinite(v.weight) && v.weight > 0)
    .sort((a, b) => b.weight - a.weight || a.index - b.index);
  const total = entries.reduce((n, v) => n + v.weight, 0);
  if (!total) return result;
  const width = 100 * (Number.isFinite(aspect) && aspect > 0 ? aspect : 1.5);
  const items = entries.map((e) => ({
    ...e,
    area: (e.weight / total) * width * 100,
  }));
  let x = 0,
    y = 0,
    w = width,
    h = 100;
  let row: typeof items = [];
  function worst(candidate: typeof items, side: number) {
    const sum = candidate.reduce((n, i) => n + i.area, 0);
    const min = candidate[candidate.length - 1].area;
    const max = candidate[0].area;
    return Math.max(
      (side * side * max) / (sum * sum),
      (sum * sum) / (side * side * min),
    );
  }
  function place() {
    const area = row.reduce((n, i) => n + i.area, 0);
    const vertical = w >= h;
    const thickness = area / (vertical ? h : w);
    let cursor = vertical ? y : x;
    for (const item of row) {
      const length = item.area / thickness;
      result[item.index] = {
        left: ((vertical ? x : cursor) / width) * 100,
        top: vertical ? cursor : y,
        width: ((vertical ? thickness : length) / width) * 100,
        height: vertical ? length : thickness,
      };
      cursor += length;
    }
    if (vertical) {
      x += thickness;
      w = Math.max(0, w - thickness);
    } else {
      y += thickness;
      h = Math.max(0, h - thickness);
    }
    row = [];
  }
  for (const item of items) {
    if (
      !row.length ||
      worst([...row, item], Math.min(w, h)) <= worst(row, Math.min(w, h))
    )
      row.push(item);
    else {
      place();
      row.push(item);
    }
  }
  if (row.length) place();
  return result;
}

/** Stable path colors identify the same object in the ranked list and map. */
export function storageColor(path: string) {
  let hash = 0;
  for (const c of path) hash = ((hash << 5) - hash + c.charCodeAt(0)) | 0;
  return (hash >>> 0) % 7;
}
