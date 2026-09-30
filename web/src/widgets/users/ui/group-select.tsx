"use client";

import { useTranslations } from "next-intl";

import { ADMIN_GROUP, givable, useGroups } from "@/entities/user";
import { useSession } from "@/entities/session";

export const NO_GROUP = "";

export const SELECT =
  "h-9 w-full rounded-md border border-input bg-glass-tint px-3 text-sm shadow-xs outline-none focus-visible:border-ring focus-visible:ring-[3px] focus-visible:ring-ring/50";

export type GroupSelectProps = { id: string; value: string; onChange: (group: string) => void };

export function GroupSelect({ id, value, onChange }: GroupSelectProps) {
  const t = useTranslations("users");
  const session = useSession().data;
  const groups = useGroups().data?.data;
  const names = groups && session ? givable(groups, session.rights, session.admin) : [];
  const offered = value === NO_GROUP || names.includes(value) ? names : [...names, value];
  return (
    <select id={id} className={SELECT} value={value} onChange={(event) => onChange(event.target.value)}>
      <option value={NO_GROUP}>{t("noGroup")}</option>
      {offered.map((name) => (
        <option key={name} value={name}>
          {name === ADMIN_GROUP ? t("adminGroup") : name}
        </option>
      ))}
    </select>
  );
}

export function groupOf(value: string): string | null {
  return value === NO_GROUP ? null : value;
}
