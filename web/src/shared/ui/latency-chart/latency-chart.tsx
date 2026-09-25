import { AXIS_GUTTER, TimeAxis, valueTicks } from "../time-axis";

export type LatencySample = { at: string; value: number | null; failed: boolean; label: string };

export type LatencyChartProps = {
  samples: LatencySample[];
  title: string;
  empty: string;
  from: number;
  to: number;
  formatTime: (at: number) => string;
  formatValue: (milliseconds: number) => string;
};

const WIDTH = 600;
const HEIGHT = 120;
const GAP_FACTOR = 3;

type Plotted = { x: number; y: number | null; failed: boolean; label: string };

export function plot(samples: LatencySample[], from: number, to: number, top: number): { points: Plotted[]; segments: string[] } {
  if (samples.length === 0) {
    return { points: [], segments: [] };
  }
  const times = samples.map((sample) => Date.parse(sample.at));
  const span = Math.max(1, to - from);
  const ceiling = Math.max(1, top);
  const points = samples.map((sample, index) => ({
    x: ((times[index] - from) / span) * WIDTH,
    y: sample.value === null ? null : HEIGHT - (sample.value / ceiling) * HEIGHT,
    failed: sample.failed,
    label: sample.label,
  }));
  const spacings = times
    .slice(1)
    .map((time, index) => time - times[index])
    .sort((left, right) => left - right);
  const typical = spacings[Math.floor(spacings.length / 2)] ?? span;
  const segments: string[] = [];
  let current: string[] = [];
  points.forEach((point, index) => {
    const broken = index > 0 && times[index] - times[index - 1] > typical * GAP_FACTOR;
    if (point.y === null || broken) {
      if (current.length > 0) {
        segments.push(current.join(" "));
      }
      current = [];
    }
    if (point.y !== null) {
      current.push(`${point.x.toFixed(1)},${point.y.toFixed(1)}`);
    }
  });
  if (current.length > 0) {
    segments.push(current.join(" "));
  }
  return { points, segments };
}

export function LatencyChart({ samples, title, empty, from, to, formatTime, formatValue }: LatencyChartProps) {
  const highest = Math.max(0, ...samples.map((sample) => sample.value ?? 0));
  const { ticks, top } = valueTicks(highest);
  const { points, segments } = plot(samples, from, to, top);
  if (points.length === 0) {
    return <p className="py-8 text-center text-sm text-muted-foreground">{empty}</p>;
  }
  return (
    <div className={AXIS_GUTTER}>
      <div aria-hidden className="relative my-2 text-right text-[11px] text-muted-foreground tabular-nums">
        {ticks.map((tick) => (
          <span
            key={tick}
            data-value-tick={tick}
            className="absolute right-0 -translate-y-1/2 whitespace-nowrap"
            style={{ top: `${((1 - tick / top) * 100).toFixed(3)}%` }}
          >
            {formatValue(tick)}
          </span>
        ))}
      </div>
      <svg viewBox={`0 0 ${WIDTH} ${HEIGHT}`} preserveAspectRatio="none" role="img" aria-label={title} className="my-2 h-32 w-full overflow-visible">
        {ticks.map((tick) => (
          <line
            key={tick}
            x1={0}
            x2={WIDTH}
            y1={HEIGHT - (tick / top) * HEIGHT}
            y2={HEIGHT - (tick / top) * HEIGHT}
            className={tick === 0 ? "stroke-border" : "stroke-border/50"}
            strokeWidth={1}
            vectorEffect="non-scaling-stroke"
            aria-hidden
          />
        ))}
        {segments.map((segment) => (
          <polyline
            key={segment}
            points={segment}
            fill="none"
            className="stroke-primary"
            strokeWidth={2}
            strokeLinejoin="round"
            strokeLinecap="round"
            vectorEffect="non-scaling-stroke"
            data-segment=""
          />
        ))}
        {points.map((point) =>
          point.failed ? (
            <rect key={`failed-${point.x}`} x={point.x - 1.5} y={HEIGHT - 10} width={3} height={10} rx={1} className="fill-status-down" data-failed="">
              <title>{point.label}</title>
            </rect>
          ) : null,
        )}
        {points.map((point) => (
          <rect key={`hit-${point.x}`} x={point.x - 6} y={0} width={12} height={HEIGHT} fill="transparent" data-x={point.x.toFixed(1)}>
            <title>{point.label}</title>
          </rect>
        ))}
      </svg>
      <TimeAxis className="col-start-2" from={from} to={to} format={formatTime} />
    </div>
  );
}
