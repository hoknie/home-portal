"use client";

import { ArrowDown, FilePlus2 } from "lucide-react";
import { useTranslations } from "next-intl";
import type { ReactNode } from "react";

import { routes } from "@/shared/config";
import { AddressLink } from "@/shared/ui/address-link";

import { TEMPLATES, type TemplateName } from "../model/templates";

export const TEMPLATE_PARAMETER = "template";

export function templateHref(name: TemplateName) {
  return `${routes.newWorkflow}?${TEMPLATE_PARAMETER}=${name}`;
}

function Preview({ kinds }: { kinds: string[] }) {
  const help = useTranslations("workflowHelp.kinds");
  if (kinds.length === 0) {
    return <FilePlus2 className="size-6 text-muted-foreground" aria-hidden />;
  }
  return (
    <ol className="flex flex-col items-center gap-0.5" aria-hidden>
      {kinds.map((kind, index) => (
        <li key={`${kind}-${index}`} className="flex flex-col items-center gap-0.5">
          {index > 0 ? <ArrowDown className="size-3 text-muted-foreground/60" /> : null}
          <span className="rounded-md border border-glass-edge bg-background px-2 py-0.5 text-[11px]">{help(`${kind}.name` as "if.name")}</span>
        </li>
      ))}
    </ol>
  );
}

export type TemplatesGalleryProps = { onChoose?: (name: TemplateName) => void };

export function TemplatesGallery({ onChoose }: TemplatesGalleryProps) {
  const t = useTranslations("workflowHelp.templates");
  return (
    <ul className="grid gap-3 sm:grid-cols-2 lg:grid-cols-3" aria-label={t("gallery")}>
      {TEMPLATES.map((template) => {
        const content: ReactNode = (
          <>
            <span className="grid min-h-28 place-items-center rounded-lg bg-glass-tint p-3">
              <Preview kinds={template.draft.steps.map((step) => step.kind)} />
            </span>
            <span className="text-sm font-semibold">{t(`${template.name}.title` as "retry.title")}</span>
            <span className="text-xs text-muted-foreground">{t(`${template.name}.description` as "retry.description")}</span>
          </>
        );
        const className = "grid h-full gap-2 rounded-xl border border-glass-edge bg-card p-3 text-start transition-colors hover:border-primary hover:shadow-md";
        return (
          <li key={template.name}>
            {onChoose ? (
              <button type="button" className={className} onClick={() => onChoose(template.name)}>
                {content}
              </button>
            ) : (
              <AddressLink href={templateHref(template.name)} className={className}>
                {content}
              </AddressLink>
            )}
          </li>
        );
      })}
    </ul>
  );
}
