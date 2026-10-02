"use client";

import type { Announcements } from "@dnd-kit/core";
import { useTranslations } from "next-intl";

import { sectionOfSortable } from "./drag";
import type { Draft } from "./draft";

export function useAnnouncements(draft: Draft): Announcements {
  const t = useTranslations("layoutEditor");
  const sectionTitle = (id: string) => draft.sections.find((section) => section.id === id)?.title ?? t("untitled");
  const sectionPlace = (id: string) => ({ position: draft.sections.findIndex((section) => section.id === id) + 1 });
  const about = (active: string | number, over: string | number | undefined, key: "over" | "end") => {
    const section = sectionOfSortable(String(active));
    const target = over === undefined ? null : sectionOfSortable(String(over));
    return section === null || target === null ? undefined : t(`announce.section.${key}`, { title: sectionTitle(section), ...sectionPlace(target) });
  };
  return {
    onDragStart: ({ active }) => {
      const section = sectionOfSortable(String(active.id));
      return section === null ? undefined : t("announce.section.start", { title: sectionTitle(section) });
    },
    onDragOver: ({ active, over }) => about(active.id, over?.id, "over"),
    onDragEnd: ({ active, over }) => about(active.id, over?.id, "end"),
    onDragCancel: ({ active }) => {
      const section = sectionOfSortable(String(active.id));
      return section === null ? undefined : t("announce.section.cancel", { title: sectionTitle(section) });
    },
  };
}
