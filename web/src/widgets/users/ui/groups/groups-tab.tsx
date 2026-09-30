"use client";

import { Pencil, Plus, Trash2 } from "lucide-react";
import { useTranslations } from "next-intl";
import { useState } from "react";
import { toast } from "sonner";

import { ADMIN_GROUP, type Group, useDeleteGroup, useGroups } from "@/entities/user";
import { useSession } from "@/entities/session";
import { RequestError } from "@/shared/api";
import { ConfirmDialog } from "@/shared/ui/confirm-dialog";
import { type Column, DataTable } from "@/shared/ui/data-table";
import { ErrorNotice } from "@/shared/ui/error-notice";
import { Badge, Button, Skeleton } from "@/shared/ui/primitives";
import { SectionCard } from "@/shared/ui/section-card";

import { GroupDialog } from "./group-dialog";
import { MatrixEditor } from "./matrix-editor";

export type GroupsTabProps = { editable: boolean };

export function GroupsTab({ editable }: GroupsTabProps) {
  const t = useTranslations("users.groups");
  const admin = useSession().data?.admin ?? false;
  const groups = useGroups();
  const data = groups.data?.data;
  const revision = groups.data?.revision ?? null;
  const [editing, setEditing] = useState<Group | null | undefined>(undefined);
  const [removing, setRemoving] = useState<Group | null>(null);
  const [shown, setShown] = useState<string>(ADMIN_GROUP);
  const remove = useDeleteGroup();
  const writable = admin && editable;
  const confirm = async (group: Group) => {
    try {
      await remove.mutateAsync({ name: group.name, revision });
      toast.success(t("deleted", { name: group.name }));
    } catch (error) {
      toast.error(t("refused", { message: error instanceof RequestError ? error.message : String(error) }));
    }
    setRemoving(null);
  };
  const columns: Column<Group>[] = [
    {
      key: "name",
      header: t("name"),
      cell: (group) => (
        <button type="button" className="inline-flex items-center gap-2 font-medium hover:underline" onClick={() => setShown(group.name)}>
          {group.name === ADMIN_GROUP ? t("admin") : group.name}
          {group.builtin ? <Badge variant="outline">{t("builtin")}</Badge> : null}
        </button>
      ),
    },
    {
      key: "members",
      header: t("members"),
      cell: (group) => <span className="text-muted-foreground">{group.members.length > 0 ? group.members.join(", ") : t("noMembers")}</span>,
    },
    {
      key: "actions",
      header: t("actionsColumn"),
      align: "end",
      cell: (group) =>
        writable && !group.builtin ? (
          <span className="inline-flex justify-end gap-1">
            <Button type="button" variant="ghost" size="sm" onClick={() => setEditing(group)}>
              <Pencil aria-hidden />
              {t("edit")}
            </Button>
            <Button
              type="button"
              variant="ghost"
              size="sm"
              disabled={group.members.length > 0}
              title={group.members.length > 0 ? t("hasMembers") : undefined}
              onClick={() => setRemoving(group)}
            >
              <Trash2 aria-hidden />
              {t("delete")}
            </Button>
          </span>
        ) : null,
    },
  ];
  if (groups.error && !data) {
    return <ErrorNotice title={t("loadFailed")} description={groups.error.message} onRetry={() => void groups.refetch()} />;
  }
  if (!data) {
    return <Skeleton className="h-64 w-full" aria-busy="true" />;
  }
  const selected = data.groups.find((group) => group.name === shown) ?? data.groups[0];
  return (
    <div className="grid gap-6">
      {writable ? (
        <div>
          <Button type="button" onClick={() => setEditing(null)}>
            <Plus aria-hidden />
            {t("add")}
          </Button>
        </div>
      ) : null}
      <SectionCard flush>
        <DataTable columns={columns} rows={data.groups} rowKey={(group) => group.name} />
      </SectionCard>
      {selected ? (
        <SectionCard title={t("rightsOf", { name: selected.name === ADMIN_GROUP ? t("admin") : selected.name })}>
          <MatrixEditor matrix={data.matrix} rights={selected.rights} readOnly />
        </SectionCard>
      ) : null}
      {editing !== undefined ? (
        <GroupDialog
          key={editing?.name ?? "new"}
          group={editing && (data.groups.find((group) => group.name === editing.name) ?? editing)}
          matrix={data.matrix}
          revision={revision}
          open
          onOpenChange={(open) => (open ? undefined : setEditing(undefined))}
        />
      ) : null}
      <ConfirmDialog
        open={removing !== null}
        onOpenChange={(open) => (open ? undefined : setRemoving(null))}
        title={t("deleteTitle", { name: removing?.name ?? "" })}
        description={t("deleteDescription", { name: removing?.name ?? "" })}
        confirmLabel={t("delete")}
        pending={remove.isPending}
        onConfirm={() => (removing ? void confirm(removing) : undefined)}
      />
    </div>
  );
}
