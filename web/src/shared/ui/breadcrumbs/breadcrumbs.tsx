"use client";

import { ChevronRight } from "lucide-react";
import Link from "next/link";
import { useTranslations } from "next-intl";
import { useState } from "react";

import { cn } from "@/shared/lib/cn";
import { AddressLink } from "@/shared/ui/address-link";

export type Crumb = { label: string; href?: string; local?: boolean };

export type BreadcrumbsProps = { items: Crumb[] };

export const FOLD_AFTER = 3;

export function Breadcrumbs({ items }: BreadcrumbsProps) {
  const t = useTranslations("breadcrumbs");
  const [open, setOpen] = useState(false);
  const folds = items.length > FOLD_AFTER && !open;
  const hidden = (index: number) => folds && index > 0 && index < items.length - 2;
  return (
    <nav aria-label={t("label")}>
      <ol className="flex min-w-0 flex-wrap items-center gap-1 text-sm text-muted-foreground">
        {items.map((item, index) => {
          const last = index === items.length - 1;
          return (
            <li key={`${index}-${item.label}`} className={cn("flex min-w-0 items-center gap-1", hidden(index) && "hidden sm:flex")}>
              {index > 0 ? <ChevronRight className="size-3.5 shrink-0 opacity-60" aria-hidden /> : null}
              {last || !item.href ? (
                <span aria-current={last ? "page" : undefined} className={cn("truncate", last && "font-medium text-foreground")}>
                  {item.label}
                </span>
              ) : item.local ? (
                <AddressLink href={item.href} className="truncate rounded-sm hover:text-foreground hover:underline">
                  {item.label}
                </AddressLink>
              ) : (
                <Link href={item.href} className="truncate rounded-sm hover:text-foreground hover:underline">
                  {item.label}
                </Link>
              )}
              {index === 0 && folds ? (
                <button
                  type="button"
                  className="ms-1 rounded-sm px-1 hover:bg-accent sm:hidden"
                  aria-label={t("more")}
                  aria-expanded={false}
                  onClick={() => setOpen(true)}
                >
                  {t("ellipsis")}
                </button>
              ) : null}
            </li>
          );
        })}
      </ol>
    </nav>
  );
}
