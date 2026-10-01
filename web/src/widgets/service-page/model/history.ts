export function durationParts(milliseconds: number) {
  const minutes = Math.max(0, Math.round(milliseconds / 60_000));
  return { days: Math.floor(minutes / 1_440), hours: Math.floor((minutes % 1_440) / 60), minutes: minutes % 60 };
}

export function percent(ratio: number | null) {
  return ratio === null ? null : Math.floor(ratio * 1000) / 10;
}
