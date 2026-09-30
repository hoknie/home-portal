"use client";

import { useTranslations } from "next-intl";

import { type Permission, PermissionStateBadge, needsAction, subjectOf } from "@/entities/permission";
import { RelativeTime } from "@/shared/ui/relative-time";

export type PermissionRowProps = { permission: Permission; owner: string };

export function PermissionRow({ permission, owner }: PermissionRowProps) {
  const t = useTranslations("permissions");
  const subject = subjectOf(permission.code);
  const label = "name" in subject ? t(`names.${subject.kind}`, { name: subject.name }) : t(`names.${subject.kind}`);
  return (
    <li className="grid gap-1 border-b py-3 last:border-b-0" data-code={permission.code}>
      <div className="flex flex-wrap items-center justify-between gap-2">
        <span className="font-medium">{label}</span>
        <PermissionStateBadge state={permission.state} />
      </div>
      {needsAction(permission) && permission.advice ? (
        <p className="text-sm text-muted-foreground">
          {t(`advice.${permission.advice}`, { pane: t(`panes.${permission.pane}`), owner })}
        </p>
      ) : null}
      {permission.learned_at ? (
        <p className="text-xs text-muted-foreground">
          {t("learned")} <RelativeTime moment={permission.learned_at} />
        </p>
      ) : null}
    </li>
  );
}
