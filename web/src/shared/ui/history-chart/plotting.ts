import { KNOWN_STATES, type KnownState } from "../status-badge";

export type HistoryPoint = { at: number; average: number | null; minimum: number | null; maximum: number | null; state: KnownState | null };

export type Interval = HistoryPoint & { until: number };

export type Band = { start: number; end: number; state: KnownState };

export const WIDTH = 600;
export const HEIGHT = 120;
export function intervalsOf(points: readonly HistoryPoint[], step: number, to: number): Interval[] {
  return [...points].sort((left, right) => left.at - right.at).map((point) => ({ ...point, until: Math.min(to, point.at + step) }));
}

export function xOf(at: number, from: number, to: number) {
  return ((at - from) / Math.max(1, to - from)) * WIDTH;
}

function yOf(value: number, top: number) {
  return HEIGHT - (value / Math.max(1, top)) * HEIGHT;
}

function middleOf(interval: Interval) {
  return (interval.at + interval.until) / 2;
}

export function runsOf(intervals: readonly Interval[]): Interval[][] {
  const runs: Interval[][] = [];
  let current: Interval[] = [];
  intervals.forEach((interval, index) => {
    const broken = index > 0 && interval.at > intervals[index - 1].until;
    if (interval.average === null || broken) {
      if (current.length > 0) {
        runs.push(current);
      }
      current = [];
    }
    if (interval.average !== null) {
      current.push(interval);
    }
  });
  if (current.length > 0) {
    runs.push(current);
  }
  return runs;
}

type Spot = { x: number; y: number };

function tangents(spots: readonly Spot[]) {
  const slopes = spots.slice(1).map((spot, index) => (spot.y - spots[index].y) / Math.max(1e-6, spot.x - spots[index].x));
  return spots.map((_, index) => {
    if (index === 0 || index === spots.length - 1) {
      return slopes[Math.min(index, slopes.length - 1)] ?? 0;
    }
    const before = slopes[index - 1];
    const after = slopes[index];
    return before * after <= 0 ? 0 : (2 * before * after) / (before + after);
  });
}

export function smoothPath(spots: readonly Spot[]) {
  if (spots.length === 0) {
    return "";
  }
  const slope = tangents(spots);
  const first = `M${spots[0].x.toFixed(1)},${spots[0].y.toFixed(1)}`;
  if (spots.length === 1) {
    return `${first}h0.1`;
  }
  return spots.slice(1).reduce((path, spot, index) => {
    const previous = spots[index];
    const third = (spot.x - previous.x) / 3;
    const one = `${(previous.x + third).toFixed(1)},${(previous.y + slope[index] * third).toFixed(1)}`;
    const two = `${(spot.x - third).toFixed(1)},${(spot.y - slope[index + 1] * third).toFixed(1)}`;
    return `${path} C${one} ${two} ${spot.x.toFixed(1)},${spot.y.toFixed(1)}`;
  }, first);
}

export function linesOf(intervals: readonly Interval[], from: number, to: number, top: number) {
  return runsOf(intervals).map((run) => {
    const at = (interval: Interval, value: number) => ({ x: xOf(middleOf(interval), from, to), y: yOf(value, top) });
    const highs = run.map((interval) => at(interval, interval.maximum ?? interval.average ?? 0));
    const lows = run.map((interval) => at(interval, interval.minimum ?? interval.average ?? 0)).reverse();
    const area = `${smoothPath(highs)} L${lows.map((spot) => `${spot.x.toFixed(1)},${spot.y.toFixed(1)}`).join(" L")} Z`;
    const means = run.map((interval) => at(interval, interval.average ?? 0));
    const line = smoothPath(means);
    const under = `${line} L${means[means.length - 1].x.toFixed(1)},${HEIGHT} L${means[0].x.toFixed(1)},${HEIGHT} Z`;
    return { line, area, under };
  });
}

export function bandsOf(intervals: readonly Interval[]): Band[] {
  const bands: Band[] = [];
  for (const interval of intervals) {
    if (interval.state === null || !KNOWN_STATES.includes(interval.state)) {
      continue;
    }
    const last = bands.at(-1);
    if (last && last.state === interval.state && last.end === interval.at) {
      last.end = interval.until;
    } else {
      bands.push({ start: interval.at, end: interval.until, state: interval.state });
    }
  }
  return bands;
}

export function nearest(intervals: readonly Interval[], at: number) {
  let best = -1;
  intervals.forEach((interval, index) => {
    if (best < 0 || Math.abs(middleOf(interval) - at) < Math.abs(middleOf(intervals[best]) - at)) {
      best = index;
    }
  });
  return best;
}

export function markerOf(interval: Interval, from: number, to: number) {
  return xOf(middleOf(interval), from, to);
}
