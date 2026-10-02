"use client";

import { Copy, LayoutDashboard, Plus, Puzzle, Settings2, Trash2 } from "lucide-react";
import Link from "next/link";
import { useRouter, useSearchParams } from "next/navigation";
import { useTranslations } from "next-intl";
import { useState } from "react";
import { toast } from "sonner";

import { type LibraryEntry, type LibraryWidget, entryOf, useDeleteLibraryWidget, useLibrary, useSaveLibraryWidget } from "@/entities/dashboard";
import { useEnvironment } from "@/entities/environment";
import { LibraryPreview, useWidgetKinds } from "@/features/widget-board";
import { CUSTOM, GalleryDialog, LibraryDialog } from "@/features/widget-library";
import { DEFAULT_APPEARANCE } from "@/shared/api";
import { routes } from "@/shared/config";
import { useTrail } from "@/shared/lib/breadcrumbs";
import { ConfirmDialog } from "@/shared/ui/confirm-dialog";
import { cn } from "@/shared/lib/cn";
import { Badge, Button, EmptyState, ErrorNotice, ListTransition, SkeletonWidget, SURFACE } from "@/shared/ui/kit";
import { PageHeader } from "@/shared/ui/page-header";

import { copyId } from "../model/copies";

export const WIDGET_PARAMETER = "widget";

export const INTERNET = "internet";

export function WidgetLibraryScreen() {
  const t = useTranslations("widgetLibrary");
  const known = useEnvironment().data?.environments ?? [];
  const environments = known.includes(INTERNET) ? known : [...known, INTERNET];
  const trail = useTrail();
  const router = useRouter();
  const asked = useSearchParams().get(WIDGET_PARAMETER);
  const kinds = useWidgetKinds();
  const library = useLibrary();
  const save = useSaveLibraryWidget();
  const remove = useDeleteLibraryWidget();
  const widgets = library.data?.data.widgets ?? [];
  const [adding, setAdding] = useState(false);
  const [editing, setEditing] = useState<{ entry: LibraryEntry } | null>(null);
  const [deleting, setDeleting] = useState<LibraryWidget | null>(null);
  const [opened, setOpened] = useState(false);
  if (!opened && asked !== null && library.data) {
    setOpened(true);
    const found = widgets.find((widget) => widget.id === asked);
    if (found?.type === CUSTOM) {
      router.replace(routes.editWidget(found.id), { scroll: false });
    } else if (found) {
      setEditing({ entry: entryOf(found) });
    }
  }
  const open = (widget: LibraryWidget) => (widget.type === CUSTOM ? router.push(routes.editWidget(widget.id)) : setEditing({ entry: entryOf(widget) }));
  const kindOf = (type: string) => kinds.find((kind) => kind.type === type);
  const nameOf = (widget: LibraryWidget) => widget.title ?? kindOf(widget.type)?.title ?? widget.id;
  const close = () => {
    setEditing(null);
    if (asked !== null) {
      router.replace(routes.adminLibrary, { scroll: false });
    }
  };
  const duplicate = async (widget: LibraryWidget) => {
    const entry = { ...entryOf(widget), id: copyId(widget.id, widgets.map((candidate) => candidate.id)) };
    await save.mutateAsync({ entry, existing: null, revision: library.data?.revision ?? null });
    toast.success(t("saved"));
  };
  const add = (
    <Button onClick={() => setAdding(true)}>
      <Plus aria-hidden />
      {t("add")}
    </Button>
  );
  return (
    <div className="grid gap-8">
      <PageHeader
        breadcrumbs={trail.of(trail.section("layout"), { label: t("title") })}
        title={t("title")}
        description={t("subtitle")}
        actions={
          <div className="flex gap-2">
            <Button asChild variant="outline">
              <Link href={routes.adminLayout}>
                <LayoutDashboard aria-hidden />
                {t("layout")}
              </Link>
            </Button>
            {add}
          </div>
        }
      />
      {library.error && !library.data ? <ErrorNotice title={t("title")} description={library.error.message} onRetry={() => void library.refetch()} /> : null}
      {!library.data && !library.error ? (
        <div className="grid gap-4 md:grid-cols-2 xl:grid-cols-3" data-skeleton="library" aria-busy="true">
          <SkeletonWidget rows={3} />
          <SkeletonWidget rows={3} />
          <SkeletonWidget rows={3} />
        </div>
      ) : null}
      {library.data && widgets.length === 0 ? <EmptyState icon={Puzzle} title={t("empty")} description={t("emptyHint")} action={add} /> : null}
      <ul className="grid gap-4 md:grid-cols-2 xl:grid-cols-3" aria-label={t("title")}>
        <ListTransition items={widgets} keyOf={(widget) => widget.id}>
          {(widget) => (
            <li className={cn(SURFACE.panel, "grid content-start gap-3 rounded-2xl p-3 transition-colors hover:border-primary/40")} data-library-widget={widget.id}>
              <div className="flex items-center gap-2">
                <div className="min-w-0 flex-1">
                  <p className="truncate text-sm font-medium">{nameOf(widget)}</p>
                  <p className="truncate font-mono text-xs text-muted-foreground">
                    {t("meta", { id: widget.id, type: kindOf(widget.type)?.title ?? widget.type })}
                  </p>
                </div>
                <Badge variant="outline">{t("placed", { placed: widget.placed })}</Badge>
              </div>
              <div inert className="pointer-events-none max-h-80 overflow-hidden rounded-xl bg-muted/40 p-3" data-library-stage="">
                <LibraryPreview widget={widget} />
              </div>
              <div className="flex gap-1">
                <Button variant="ghost" size="sm" onClick={() => open(widget)} aria-label={`${t("edit")}: ${nameOf(widget)}`}>
                  <Settings2 aria-hidden />
                  {t("edit")}
                </Button>
                <Button variant="ghost" size="icon" aria-label={`${t("duplicate")}: ${nameOf(widget)}`} title={t("duplicate")} onClick={() => void duplicate(widget)}>
                  <Copy aria-hidden />
                </Button>
                <Button
                  variant="ghost"
                  size="icon"
                  className="ml-auto"
                  aria-label={`${t("delete")}: ${nameOf(widget)}`}
                  title={widget.placed > 0 ? t("usedHint") : t("delete")}
                  disabled={widget.placed > 0}
                  onClick={() => setDeleting(widget)}
                >
                  <Trash2 aria-hidden />
                </Button>
              </div>
            </li>
          )}
        </ListTransition>
      </ul>
      <GalleryDialog
        open={adding}
        kinds={kinds}
        onClose={() => setAdding(false)}
        onChoose={(choice) => {
          setAdding(false);
          if (choice.type === CUSTOM) {
            router.push(routes.newWidget(choice.template));
          } else {
            setEditing({ entry: { id: null, type: choice.type, title: null, settings: choice.settings, environments: null, public: false, appearance: DEFAULT_APPEARANCE } });
          }
        }}
      />
      <LibraryDialog
        entry={editing?.entry ?? null}
        kinds={kinds}
        environments={environments}
        renderPreview={(entry, custom) => <LibraryPreview widget={entry} custom={custom} />}
        onSaved={() => toast.success(t("saved"))}
        onClose={close}
      />
      <ConfirmDialog
        open={deleting !== null}
        title={deleting ? t("deleteTitle", { title: nameOf(deleting) }) : ""}
        description={t("deleteDescription")}
        confirmLabel={t("delete")}
        pending={remove.isPending}
        onOpenChange={(open) => (open ? undefined : setDeleting(null))}
        onConfirm={() => {
          if (deleting) {
            remove.mutate(
              { id: deleting.id, revision: library.data?.revision ?? null },
              { onSuccess: () => toast.success(t("deleted")), onError: (error) => toast.error(error.message), onSettled: () => setDeleting(null) },
            );
          }
        }}
      />
    </div>
  );
}
