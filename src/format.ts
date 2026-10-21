export function bytes(value: number) {
  if (!Number.isFinite(value) || value < 0) return 'Unavailable';
  if (value === 0) return '0 B';
  const units = ['B', 'KB', 'MB', 'GB', 'TB', 'PB'];
  const level = Math.min(
    Math.floor(Math.log(value) / Math.log(1000)),
    units.length - 1,
  );
  return `${(value / 1000 ** level).toLocaleString(undefined, { maximumFractionDigits: level > 1 ? 1 : 0 })} ${units[level]}`;
}
export function date(value: string | null) {
  if (!value) return 'Not recorded';
  const d = new Date(value);
  return Number.isNaN(d.valueOf())
    ? 'Unavailable'
    : d.toLocaleString(undefined, {
        month: 'short',
        day: 'numeric',
        hour: 'numeric',
        minute: '2-digit',
      });
}
export function label(value: string) {
  return value.replace(/([A-Z])/g, ' $1').replace(/^./, (c) => c.toUpperCase());
}
export const workloads = [
  'Gaming',
  'Software development',
  'Video editing',
  'Graphic design',
  'Music production',
  'General productivity',
  'School or research',
  'Virtual machines and containers',
  'Other / custom',
];
