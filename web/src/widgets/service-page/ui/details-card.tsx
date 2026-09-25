"use client";

import { ExternalLink } from "lucide-react";
import { useTranslations } from "next-intl";

import type { Service } from "@/entities/service";
import { SectionCard } from "@/shared/ui/section-card";

import { Notes } from "./notes";

export function DetailsCard({ service }: { service: Service }) {
  const t = useTranslations("servicePage");
  if (service.links.length === 0 && !service.notes) {
    return null;
  }
  return (
    <div className="grid gap-6 lg:grid-cols-[1fr_2fr]">
      {service.links.length > 0 ? (
        <SectionCard title={t("links")}>
          <ul className="grid gap-2">
            {service.links.map((link) => (
              <li key={`${link.title}-${link.url}`}>
                <a href={link.url} target="_blank" rel="noreferrer" className="inline-flex items-center gap-2 text-sm hover:underline">
                  <ExternalLink className="size-3.5 text-muted-foreground" aria-hidden />
                  {link.title}
                </a>
              </li>
            ))}
          </ul>
        </SectionCard>
      ) : null}
      {service.notes ? (
        <SectionCard title={t("notes")}>
          <Notes text={service.notes} />
        </SectionCard>
      ) : null}
    </div>
  );
}
