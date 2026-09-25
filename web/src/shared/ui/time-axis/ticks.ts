export const AXIS_GUTTER = "grid grid-cols-[3rem_minmax(0,1fr)] gap-x-2";

export const PIXELS_PER_LABEL = 72;

const MINUTE = 60_000;
const HOUR = 60 * MINUTE;
const DAY = 24 * HOUR;

const SUB_DAY_STEPS = [15 * MINUTE, 30 * MINUTE, HOUR, 2 * HOUR, 3 * HOUR, 6 * HOUR, 12 * HOUR];
const DAY_STEPS = [1, 2, 7, 14];

export type Tick = { at: number; position: number };

export function labelsFor(width: number) {
  return Math.max(2, Math.floor(width / PIXELS_PER_LABEL));
}

function midnight(at: number) {
  const date = new Date(at);
  date.setHours(0, 0, 0, 0);
  return date;
}

function placed(ticks: number[], from: number, to: number): Tick[] {
  const span = Math.max(1, to - from);
  return ticks.filter((at) => at >= from && at <= to).map((at) => ({ at, position: (at - from) / span }));
}

export function timeTicks(from: number, to: number, width: number): Tick[] {
  const span = to - from;
  if (!(span > 0)) {
    return [];
  }
  const most = labelsFor(width);
  const subDay = span <= DAY ? (SUB_DAY_STEPS.find((step) => span / step <= most) ?? DAY / 2) : undefined;
  if (subDay !== undefined) {
    const start = midnight(from).getTime();
    const ticks: number[] = [];
    for (let at = start + Math.ceil((from - start) / subDay) * subDay; at <= to; at += subDay) {
      ticks.push(at);
    }
    return placed(ticks, from, to);
  }
  const days = DAY_STEPS.find((step) => span / (step * DAY) <= most) ?? Math.ceil(span / DAY / most);
  const cursor = midnight(from);
  if (cursor.getTime() < from) {
    cursor.setDate(cursor.getDate() + 1);
  }
  const ticks: number[] = [];
  while (cursor.getTime() <= to) {
    ticks.push(cursor.getTime());
    cursor.setDate(cursor.getDate() + days);
  }
  return placed(ticks, from, to);
}

function niceStep(raw: number) {
  const power = 10 ** Math.floor(Math.log10(raw));
  const found = [1, 2, 5, 10].find((factor) => factor * power >= raw) ?? 10;
  return found * power;
}

export function valueTicks(highest: number): { ticks: number[]; top: number } {
  const step = Math.max(1, niceStep(Math.max(1, highest) / 3));
  const count = Math.max(2, Math.ceil(highest / step));
  const ticks = Array.from({ length: count + 1 }, (_, index) => index * step);
  return { ticks, top: count * step };
}
