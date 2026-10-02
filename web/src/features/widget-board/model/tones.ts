export function percentOf(value: number, maximum: number) {
  if (!(maximum > 0)) {
    return 0;
  }
  return Math.min(100, Math.max(0, (value / maximum) * 100));
}
