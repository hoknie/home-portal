"use client";

import { useTranslations } from "next-intl";

import { type User, useUsers } from "@/entities/user";
import { type Column, DataTable } from "@/shared/ui/data-table";
import { ErrorNotice } from "@/shared/ui/error-notice";
import { ModuleOffNotice } from "@/shared/ui/module-off-notice";
import { useTrail } from "@/shared/lib/breadcrumbs";
import { PageHeader } from "@/shared/ui/page-header";
import { Badge, Skeleton } from "@/shared/ui/primitives";
import { SectionCard } from "@/shared/ui/section-card";

import { AddUserDialog } from "./add-user-dialog";
import { DeleteUserButton } from "./delete-user-button";
import { PasswordDialog } from "./password-dialog";

export function UsersScreen() {
  const trail = useTrail();
  const t = useTranslations();
  const users = useUsers();
  const data = users.data?.data;
  const revision = users.data?.revision ?? null;
  const columns: Column<User>[] = data
    ? [
        {
          key: "name",
          header: t("users.columns.name"),
          cell: (user) => (
            <span className="inline-flex items-center gap-2 font-medium">
              {user.name}
              {user.you ? <Badge variant="outline">{t("users.you")}</Badge> : null}
            </span>
          ),
        },
        {
          key: "actions",
          header: t("users.columns.actions"),
          align: "end",
          cell: (user) => (
            <span className="inline-flex flex-wrap justify-end gap-1">
              <PasswordDialog name={user.name} revision={revision} disabled={!data.editable} />
              <DeleteUserButton user={user} users={data} revision={revision} />
            </span>
          ),
        },
      ]
    : [];
  return (
    <div className="grid gap-8">
      <PageHeader breadcrumbs={trail.of(trail.section("users"))}
        title={t("users.title")}
        description={t("users.subtitle")}
        actions={<AddUserDialog revision={revision} disabled={!data?.editable} />}
      />
      {data && !data.editable ? <ModuleOffNotice name={t("modules.names.users")} /> : null}
      {users.error && !data ? (
        <ErrorNotice title={t("errors.loadFailed")} description={users.error.message} onRetry={() => void users.refetch()} />
      ) : null}
      {data ? (
        <SectionCard flush>
          <DataTable columns={columns} rows={data.users} rowKey={(user) => user.name} />
        </SectionCard>
      ) : users.error ? null : (
        <Skeleton className="h-64 w-full" aria-busy="true" />
      )}
    </div>
  );
}
