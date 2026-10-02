"use client";

import { useTranslations } from "next-intl";

import { Allowed } from "@/entities/session";
import { usePermissions } from "@/entities/permission";
import { useTrail } from "@/shared/lib/breadcrumbs";
import { Appear, ErrorNotice, SkeletonCard } from "@/shared/ui/kit";
import { PageHeader } from "@/shared/ui/page-header";
import { SectionCard } from "@/shared/ui/section-card";

import { PermissionRow } from "./permission-row";
import { RequestPermissionsButton } from "./request-permissions-button";

export const MACOS = "macos";

export function PermissionsScreen() {
  const trail = useTrail();
  const t = useTranslations("permissions");
  const permissions = usePermissions();
  const data = permissions.data?.data;
  return (
    <div className="grid gap-8">
      <PageHeader
        breadcrumbs={trail.of(trail.section("permissions"))}
        title={t("title")}
        description={t("subtitle")}
        actions={
          data?.platform === MACOS ? (
            <Allowed area="host-permissions" action="update">
              <RequestPermissionsButton />
            </Allowed>
          ) : null
        }
      />
      {permissions.error && !data ? (
        <ErrorNotice title={t("loadFailed")} description={permissions.error.message} onRetry={() => void permissions.refetch()} />
      ) : null}
      {data ? (
        <Appear>
          <SectionCard
            title={t("listTitle")}
            description={data.platform === MACOS ? t(`owner.${data.owner.kind}`, { name: data.owner.name }) : t("elsewhere")}
          >
            <ul className="grid">
              {data.permissions.map((permission) => (
                <PermissionRow key={permission.code} permission={permission} owner={data.owner.name} />
              ))}
            </ul>
          </SectionCard>
        </Appear>
      ) : permissions.error ? null : (
        <SkeletonCard lines={6} />
      )}
    </div>
  );
}
