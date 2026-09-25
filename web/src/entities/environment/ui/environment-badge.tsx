"use client";

import { Globe, House, ShieldCheck } from "lucide-react";
import { useTranslations } from "next-intl";
import type { ReactNode } from "react";

export type EnvironmentBadgeProps = { name: string | null };

const ICONS: Record<string, ReactNode> = {
  internet: <Globe className="size-3.5" aria-hidden />,
  vpn: <ShieldCheck className="size-3.5" aria-hidden />,
};

const DEFAULT_ICON = <House className="size-3.5" aria-hidden />;

export function iconFor(name: string): ReactNode {
  return ICONS[name] ?? DEFAULT_ICON;
}

export function EnvironmentBadge({ name }: EnvironmentBadgeProps) {
  const t = useTranslations("environment");
  if (!name) {
    return null;
  }
  return (
    <span
      data-environment={name}
      title={t("explains")}
      className="inline-flex items-center gap-1.5 rounded-full border border-glass-edge bg-glass-tint px-2.5 py-1 text-xs text-muted-foreground"
    >
      {iconFor(name)}
      <span className="sr-only">{t("label")}</span>
      {name}
    </span>
  );
}
