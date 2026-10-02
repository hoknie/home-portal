"use client";

import { Check, Layers } from "lucide-react";
import { useTranslations } from "next-intl";
import type { ReactNode } from "react";

import { cn } from "@/shared/lib/cn";
import { BareButton } from "@/shared/ui/kit";

import { iconFor } from "./environment-badge";

export type EnvironmentPickerProps = {
  label: string;
  environments: string[];
  value: string[] | null;
  current?: string | null;
  onChange: (value: string[] | null) => void;
};

function Tile({ chosen, icon, title, note, onClick }: { chosen: boolean; icon: ReactNode; title: string; note: string | null; onClick: () => void }) {
  return (
    <BareButton
      role="checkbox"
      aria-checked={chosen}
      onClick={onClick}
      className={cn(
        "relative flex min-w-0 items-center gap-3 rounded-xl border p-3 text-left transition-colors outline-none focus-visible:ring-[3px] focus-visible:ring-ring/50",
        chosen ? "border-primary bg-primary/10" : "border-glass-edge bg-glass-tint hover:border-primary/50",
      )}
    >
      <span className={cn("grid size-9 shrink-0 place-items-center rounded-lg [&>svg]:size-4", chosen ? "bg-primary text-primary-foreground" : "bg-muted text-muted-foreground")}>{icon}</span>
      <span className="grid min-w-0 flex-1">
        <span className="truncate text-sm font-medium">{title}</span>
        {note ? <span className="truncate text-xs text-muted-foreground">{note}</span> : null}
      </span>
      <span aria-hidden className={cn("grid size-5 shrink-0 place-items-center rounded-full border", chosen ? "border-primary bg-primary text-primary-foreground" : "border-glass-edge")}>
        {chosen ? <Check className="size-3" /> : null}
      </span>
    </BareButton>
  );
}

export function EnvironmentPicker({ label, environments, value, current = null, onChange }: EnvironmentPickerProps) {
  const t = useTranslations("environment.picker");
  const chosen = value ?? [];
  const toggle = (name: string) => {
    const next = chosen.includes(name) ? chosen.filter((candidate) => candidate !== name) : [...chosen, name];
    onChange(next.length === 0 ? null : next);
  };
  return (
    <div role="group" aria-label={label} className="@container grid gap-2" data-environment-picker="">
      <div className="grid gap-2 @sm:grid-cols-2">
        <Tile chosen={value === null} icon={<Layers aria-hidden />} title={t("everywhere")} note={t("everywhereNote")} onClick={() => onChange(null)} />
        {environments.map((name) => (
          <Tile key={name} chosen={chosen.includes(name)} icon={iconFor(name)} title={name} note={name === current ? t("here") : t(name === "internet" ? "outside" : "inside")} onClick={() => toggle(name)} />
        ))}
      </div>
    </div>
  );
}
