"use client";

import { useTranslations } from "next-intl";

import { type Metrics } from "@/entities/widget";
import { MetricBar } from "@/shared/ui/metric-bar";
import { StatTile } from "@/shared/ui/stat-tile";

export type HostMetricsProps = { data: Metrics };

const GIBIBYTE = 1024 ** 3;

export function gibibytes(bytes: number) {
  return Math.round((bytes / GIBIBYTE) * 10) / 10;
}

export function hours(seconds: number) {
  return Math.floor(seconds / 3600);
}

export function HostMetricsWidget({ data }: HostMetricsProps) {
  const t = useTranslations("widgets.metrics");
  const memory = (data.memory.used_bytes / Math.max(1, data.memory.total_bytes)) * 100;
  return (
    <div className="grid gap-4 rounded-xl border bg-card p-4">
      <div className="grid gap-2 @md:grid-cols-2">
        <MetricBar label={t("cpu")} value={`${Math.round(data.cpu_percent)}%`} percent={data.cpu_percent} />
        <MetricBar
          label={t("memory")}
          value={t("ofTotal", { used: gibibytes(data.memory.used_bytes), total: gibibytes(data.memory.total_bytes) })}
          percent={memory}
        />
      </div>
      <div className="grid gap-2 @md:grid-cols-2">
        {data.disks.map((disk) => {
          const used = disk.total_bytes - disk.available_bytes;
          return (
            <MetricBar
              key={disk.mount_point}
              label={disk.mount_point}
              value={t("ofTotal", { used: gibibytes(used), total: gibibytes(disk.total_bytes) })}
              percent={(used / Math.max(1, disk.total_bytes)) * 100}
            />
          );
        })}
      </div>
      <div className="grid grid-cols-2 gap-3 @sm:grid-cols-3">
        <StatTile label={t("uptime")} value={t("hours", { value: hours(data.uptime_seconds) })} />
        <StatTile label={t("host")} value={data.hostname ?? t("unknownHost")} />
        {data.load_average ? <StatTile label={t("load")} value={data.load_average[0].toFixed(2)} /> : null}
      </div>
    </div>
  );
}
