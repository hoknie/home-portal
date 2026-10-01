"use client";

import { ChevronLeft, ChevronRight } from "lucide-react";
import { useTranslations } from "next-intl";

import { Button } from "@/shared/ui/primitives";

import type { RunPages } from "../model/run-pages";

export function RunPager({ pages, nextBefore }: { pages: RunPages; nextBefore: string | null }) {
  const t = useTranslations("runPager");
  if (pages.page === 1 && nextBefore === null) {
    return null;
  }
  return (
    <div className="flex flex-wrap items-center justify-between gap-2 border-t border-glass-edge px-4 py-2 text-sm">
      <span className="text-muted-foreground">
        {pages.page > 1 ? (
          <>
            {t("frozen", { page: pages.page })}{" "}
            <button type="button" className="underline" onClick={pages.newest}>
              {t("newest")}
            </button>
          </>
        ) : (
          t("page", { page: pages.page })
        )}
      </span>
      <span className="flex gap-1">
        <Button variant="ghost" size="sm" disabled={pages.page === 1} onClick={pages.newer}>
          <ChevronLeft aria-hidden />
          {t("newer")}
        </Button>
        <Button variant="ghost" size="sm" disabled={nextBefore === null} onClick={() => nextBefore !== null && pages.older(nextBefore)}>
          {t("older")}
          <ChevronRight aria-hidden />
        </Button>
      </span>
    </div>
  );
}
