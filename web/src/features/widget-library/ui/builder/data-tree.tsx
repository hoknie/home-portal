"use client";

import { Play } from "lucide-react";
import { useTranslations } from "next-intl";

import { Button } from "@/shared/ui/primitives";

import type { KnownPath } from "../../model/suggestion-source";

export type DataTreeProps = { known: KnownPath[]; running: boolean; canRun: boolean; onRun: () => void; onInsert: (path: string) => void };

export function DataTree({ known, running, canRun, onRun, onInsert }: DataTreeProps) {
  const t = useTranslations("widgetBuilder.tree");
  return (
    <section aria-labelledby="builder-data-tree" className="grid gap-2 rounded-xl border border-glass-edge bg-muted/30 p-3" data-data-tree="">
      <div className="flex items-center gap-2">
        <h3 id="builder-data-tree" className="mr-auto text-sm font-medium">
          {t("title")}
        </h3>
        {canRun ? (
          <Button type="button" variant="outline" size="sm" onClick={onRun} disabled={running}>
            <Play aria-hidden />
            {t("runNow")}
          </Button>
        ) : null}
      </div>
      {known.length === 0 ? <p className="text-xs text-muted-foreground">{t("empty")}</p> : null}
      <ul className="grid max-h-56 gap-0.5 overflow-auto">
        {known.map((path) => (
          <li key={path.path}>
            <button
              type="button"
              onMouseDown={(event) => event.preventDefault()}
              onClick={() => onInsert(path.path)}
              aria-label={t("insert", { path: path.path })}
              className="flex w-full items-baseline gap-2 rounded px-1 py-0.5 text-left text-xs hover:bg-muted"
            >
              <span className="font-mono">{path.path}</span>
              <span className="text-muted-foreground">{t(`kinds.${path.kind}` as "kinds.value")}</span>
              {path.sample !== null ? <span className="ml-auto truncate text-muted-foreground">{path.sample}</span> : null}
              {path.description !== null ? <span className="basis-full text-muted-foreground">{path.description}</span> : null}
            </button>
          </li>
        ))}
      </ul>
    </section>
  );
}
