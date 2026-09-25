"use client";

import { House } from "lucide-react";
import Link from "next/link";
import { usePathname } from "next/navigation";
import { useTranslations } from "next-intl";

import { routes } from "@/shared/config";
import { cn } from "@/shared/lib/cn";

import { NAVIGATION, isActive } from "../model/navigation";

export type NavLinksProps = { onNavigate?: () => void };

export function NavLinks({ onNavigate }: NavLinksProps) {
  const t = useTranslations("nav");
  const pathname = usePathname();
  return (
    <nav className="grid gap-1">
      <Link
        href={routes.home}
        onClick={onNavigate}
        className="mb-2 flex items-center gap-3 rounded-lg px-3 py-2 text-sm font-medium text-sidebar-foreground/80 transition-colors hover:bg-sidebar-accent/60"
      >
        <House className="size-4" aria-hidden />
        {t("backHome")}
      </Link>
      {NAVIGATION.map(({ href, label, icon: Icon }) => {
        const active = isActive(pathname, href);
        return (
          <Link
            key={href}
            href={href}
            onClick={onNavigate}
            aria-current={active ? "page" : undefined}
            className={cn(
              "flex items-center gap-3 rounded-lg px-3 py-2 text-sm font-medium transition-colors",
              active ? "bg-sidebar-accent text-sidebar-accent-foreground" : "text-sidebar-foreground/80 hover:bg-sidebar-accent/60",
            )}
          >
            <Icon className="size-4" aria-hidden />
            {t(label)}
          </Link>
        );
      })}
    </nav>
  );
}
