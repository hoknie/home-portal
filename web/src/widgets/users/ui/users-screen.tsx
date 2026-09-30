"use client";

import { useTranslations } from "next-intl";
import { useState } from "react";

import { useCan, useSession } from "@/entities/session";
import { ADMIN_GROUP, type User, type Users, useUsers } from "@/entities/user";
import { type Column, DataTable } from "@/shared/ui/data-table";
import { ErrorNotice } from "@/shared/ui/error-notice";
import { ModuleOffNotice } from "@/shared/ui/module-off-notice";
import { useTrail } from "@/shared/lib/breadcrumbs";
import { PageHeader } from "@/shared/ui/page-header";
import { Badge, Button, Skeleton } from "@/shared/ui/primitives";
import { SectionCard } from "@/shared/ui/section-card";

import { AddUserDialog } from "./add-user-dialog";
import { ChangeGroupDialog } from "./change-group-dialog";
import { DeleteUserButton } from "./delete-user-button";
import { GroupsTab } from "./groups/groups-tab";
import { PasswordDialog } from "./password-dialog";

type Tab = "users" | "groups";

const TABS: Tab[] = ["users", "groups"];

function useColumns(data: Users | undefined, revision: string | null): Column<User>[] {
  const t = useTranslations();
  const can = useCan();
  const admin = useSession().data?.admin ?? false;
  if (!data) {
    return [];
  }
  const guarded = (user: User) => user.group === ADMIN_GROUP && !admin;
  return [
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
      key: "group",
      header: t("users.columns.group"),
      cell: (user) => (
        <span className={user.group === null ? "text-muted-foreground" : undefined}>
          {user.group === null ? t("users.noGroup") : user.group === ADMIN_GROUP ? t("users.adminGroup") : user.group}
        </span>
      ),
    },
    {
      key: "actions",
      header: t("users.columns.actions"),
      align: "end",
      cell: (user) =>
        guarded(user) && !user.you ? null : (
          <span className="inline-flex flex-wrap justify-end gap-1">
            {can("users", "update") && !guarded(user) ? <ChangeGroupDialog user={user} revision={revision} disabled={!data.editable} /> : null}
            {user.you || can("users", "update") ? <PasswordDialog name={user.name} revision={revision} disabled={!data.editable} /> : null}
            {can("users", "delete") && !guarded(user) ? <DeleteUserButton user={user} users={data} revision={revision} /> : null}
          </span>
        ),
    },
  ];
}

export function UsersScreen() {
  const trail = useTrail();
  const t = useTranslations();
  const can = useCan();
  const users = useUsers();
  const data = users.data?.data;
  const revision = users.data?.revision ?? null;
  const columns = useColumns(data, revision);
  const [tab, setTab] = useState<Tab>("users");
  return (
    <div className="grid gap-8">
      <PageHeader
        breadcrumbs={trail.of(trail.section("users"))}
        title={t("users.title")}
        description={t("users.subtitle")}
        actions={tab === "users" && can("users", "create") ? <AddUserDialog revision={revision} disabled={!data?.editable} /> : null}
      />
      {data && !data.editable ? <ModuleOffNotice name={t("modules.names.users")} /> : null}
      <div role="tablist" aria-label={t("users.tabs.label")} className="flex gap-2">
        {TABS.map((name) => (
          <Button key={name} type="button" role="tab" aria-selected={tab === name} variant={tab === name ? "secondary" : "ghost"} size="sm" onClick={() => setTab(name)}>
            {t(`users.tabs.${name}`)}
          </Button>
        ))}
      </div>
      {tab === "groups" ? (
        <GroupsTab editable={data?.editable ?? false} />
      ) : (
        <>
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
        </>
      )}
    </div>
  );
}
