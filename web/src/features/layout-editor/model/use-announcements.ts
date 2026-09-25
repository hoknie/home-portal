"use client";

import type { Announcements } from "@dnd-kit/core";
import { useTranslations } from "next-intl";

import { sectionOfSortable } from "./drag";
import { type Draft, widgetsOf } from "./draft";

export function useAnnouncements(draft: Draft, titleOf: (uid: string) => string): Announcements {
  const t = useTranslations("layoutEditor");
  const sectionTitle = (id: string) => draft.sections.find((section) => section.id === id)?.title ?? t("untitled");
  const sectionPlace = (id: string) => ({ position: draft.sections.findIndex((section) => section.id === id) + 1 });
  const place = (uid: string) => {
    const widget = draft.widgets.find((candidate) => candidate.uid === uid);
    const section = draft.sections.find((candidate) => candidate.id === widget?.section);
    const position = widget ? widgetsOf(draft, widget.section).findIndex((candidate) => candidate.uid === uid) + 1 : 0;
    return { position, section: section?.title ?? t("untitled") };
  };
  const about = (active: string | number, over: string | number | undefined, key: "over" | "end") => {
    const section = sectionOfSortable(String(active));
    if (section !== null) {
      const target = over === undefined ? null : sectionOfSortable(String(over));
      return target === null ? undefined : t(`announce.section.${key}`, { title: sectionTitle(section), ...sectionPlace(target) });
    }
    return over === undefined ? undefined : t(`announce.${key}`, { title: titleOf(String(active)), ...place(String(over)) });
  };
  return {
    onDragStart: ({ active }) => {
      const section = sectionOfSortable(String(active.id));
      return section === null ? t("announce.start", { title: titleOf(String(active.id)) }) : t("announce.section.start", { title: sectionTitle(section) });
    },
    onDragOver: ({ active, over }) => about(active.id, over?.id, "over"),
    onDragEnd: ({ active, over }) => about(active.id, over?.id, "end"),
    onDragCancel: ({ active }) => {
      const section = sectionOfSortable(String(active.id));
      return section === null ? t("announce.cancel", { title: titleOf(String(active.id)) }) : t("announce.section.cancel", { title: sectionTitle(section) });
    },
  };
}
