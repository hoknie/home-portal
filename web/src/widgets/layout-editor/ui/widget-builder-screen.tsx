"use client";

import { useQueryClient } from "@tanstack/react-query";
import { useRouter, useSearchParams } from "next/navigation";
import { useTranslations } from "next-intl";
import { useState } from "react";

import { type LibraryEntry, entryOf, fetchLayout, layoutKey, useLayout, useLibrary, useSaveLayout } from "@/entities/dashboard";
import { isSavedPlace, resizeKept, resizedLayout } from "@/features/layout-editor";
import { useEnvironment } from "@/entities/environment";
import { LibraryPreview, useWidgetKinds } from "@/features/widget-board";
import { CUSTOM, type CanvasSize, WidgetBuilder, useTemplateSettings } from "@/features/widget-library";
import { DEFAULT_APPEARANCE } from "@/shared/api";
import { routes } from "@/shared/config";
import { ErrorNotice } from "@/shared/ui/error-notice";
import { Skeleton } from "@/shared/ui/primitives";

import { INTERNET } from "./widget-library-screen";

export const DEFAULT_SIZE: CanvasSize = { width: 4, height: "auto" };

export function WidgetBuilderScreen({ mode }: { mode: "new" | "edit" }) {
  const t = useTranslations();
  const router = useRouter();
  const client = useQueryClient();
  const params = useSearchParams();
  const id = params.get("id");
  const back = params.get("back") === "layout" ? "layout" : "library";
  const backTo = back === "layout" ? routes.adminLayout : routes.adminLibrary;
  const kinds = useWidgetKinds();
  const library = useLibrary();
  const layout = useLayout();
  const saveLayout = useSaveLayout();
  const known = useEnvironment().data?.environments ?? [];
  const environments = known.includes(INTERNET) ? known : [...known, INTERNET];
  const templateSettings = useTemplateSettings(params.get("template"));
  const [reloads, setReloads] = useState(0);
  const found = id === null ? undefined : library.data?.data.widgets.find((widget) => widget.id === id);
  const places = (layout.data?.data.widgets ?? []).filter((widget) => widget.id === id);
  const place = params.get("place") ?? (places.length === 1 ? places[0].key : null);
  const placement = places.find((widget) => widget.key === place) ?? places[0];
  const size: CanvasSize = placement ? { width: placement.width, height: placement.height } : found ? { width: found.width, height: found.height } : DEFAULT_SIZE;
  const fresh: LibraryEntry = { id: null, type: CUSTOM, title: null, settings: templateSettings, environments: null, public: false, appearance: DEFAULT_APPEARANCE };
  const entry = mode === "new" ? fresh : found ? entryOf(found) : null;
  if (!library.data || (mode === "edit" && !layout.data && !layout.error)) {
    return library.error ? <ErrorNotice title={t("errors.loadFailed")} description={library.error.message} onRetry={() => void library.refetch()} /> : <Skeleton className="h-96 w-full" aria-busy="true" />;
  }
  if (entry === null) {
    return <ErrorNotice title={t("errors.loadFailed")} description={id ?? ""} />;
  }
  const keepSize = async (wanted: CanvasSize): Promise<string | null> => {
    if (place === null || (placement && placement.width === wanted.width && placement.height === wanted.height)) {
      return null;
    }
    if (!isSavedPlace(place)) {
      resizeKept(place, wanted, null);
      return null;
    }
    const current = await fetchLayout();
    const saved = await saveLayout.mutateAsync({ layout: resizedLayout(current.data, place, wanted), revision: current.revision });
    resizeKept(place, wanted, saved.data);
    return saved.revision;
  };
  return (
    <WidgetBuilder
      key={`${mode}-${id ?? "new"}-${reloads}`}
      entry={entry}
      revision={library.data.revision}
      isNew={mode === "new"}
      kind={kinds.find((kind) => kind.type === entry.type) ?? null}
      environments={environments}
      initialSize={size}
      renderCanvas={(edited, custom, marking) => <LibraryPreview widget={edited} custom={custom} marking={marking} />}
      onSaved={async (saved, _revision, wanted) => {
        const revision = await keepSize(wanted);
        if (mode === "new") {
          router.replace(routes.editWidget(saved, back), { scroll: false });
        }
        return revision;
      }}
      onReload={() => void library.refetch().then(() => setReloads((count) => count + 1))}
      onBack={() => {
        if (back === "layout") {
          client.removeQueries({ queryKey: layoutKey });
        }
        router.push(backTo);
      }}
    />
  );
}
