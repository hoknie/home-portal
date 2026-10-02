"use client";

import { useRouter } from "next/navigation";
import { useTranslations } from "next-intl";

import { entryOf, useLibrary } from "@/entities/dashboard";
import { useEnvironment } from "@/entities/environment";
import { LayoutEditor } from "@/features/layout-editor";
import { LibraryPreview, useWidgetKinds } from "@/features/widget-board";
import { CUSTOM, LibraryDialog } from "@/features/widget-library";
import { routes } from "@/shared/config";
import { useTrail } from "@/shared/lib/breadcrumbs";
import { PageHeader } from "@/shared/ui/page-header";

export const INTERNET = "internet";


export function LayoutEditorScreen() {
  const trail = useTrail();
  const router = useRouter();
  const t = useTranslations();
  const kinds = useWidgetKinds();
  const known = useEnvironment().data?.environments ?? [];
  const environments = known.includes(INTERNET) ? known : [...known, INTERNET];
  const library = useLibrary().data?.data.widgets ?? [];
  return (
    <div className="grid gap-8">
      <PageHeader breadcrumbs={trail.of(trail.section("layout"))} title={t("layoutEditor.title")} description={t("layoutEditor.subtitle")} />
      <p className="-mt-4 text-sm text-muted-foreground">{t("layoutEditor.examples")}</p>
      <LayoutEditor
        kinds={kinds}
        builds={(type) => type === CUSTOM}
        onBuild={(id, place) => router.push(routes.editWidget(id, "layout", place))}
        renderWidget={(widget, placement) => <LibraryPreview widget={widget} height={placement.height} />}
        renderSettings={(id, close, adopt) => {
          const found = library.find((widget) => widget.id === id);
          return (
            <LibraryDialog
              entry={found ? entryOf(found) : null}
              kinds={kinds}
              environments={environments}
              renderPreview={(entry, custom) => <LibraryPreview widget={entry} custom={custom} />}
              onSaved={(revision) => adopt(revision)}
              onClose={close}
            />
          );
        }}
      />
    </div>
  );
}
