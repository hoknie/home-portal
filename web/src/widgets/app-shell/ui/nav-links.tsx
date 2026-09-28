"use client";

import type { LucideIcon } from "lucide-react";
import Link from "next/link";
import { usePathname } from "next/navigation";
import { useTranslations } from "next-intl";

import { enabledModules, useModules } from "@/entities/module";
import { cn } from "@/shared/lib/cn";
import { Separator, Tooltip, TooltipContent, TooltipTrigger } from "@/shared/ui/primitives";

import { MANAGEMENT, isActive, moduleLinks } from "../model/navigation";

export type NavLinksProps = { onNavigate?: () => void; compact?: boolean };

type NavItemProps = { href: string; label: string; icon: LucideIcon; pathname: string; onNavigate?: () => void; compact?: boolean };

function NavItem({ href, label, icon: Icon, pathname, onNavigate, compact = false }: NavItemProps) {
  const active = isActive(pathname, href);
  const link = (
    <Link
      href={href}
      onClick={onNavigate}
      aria-current={active ? "page" : undefined}
      aria-label={compact ? label : undefined}
      className={cn(
        "flex items-center gap-3 rounded-lg py-2 text-sm font-medium transition-colors",
        compact ? "justify-center px-2" : "px-3",
        active ? "bg-sidebar-accent text-sidebar-accent-foreground" : "text-sidebar-foreground/80 hover:bg-sidebar-accent/60",
      )}
    >
      <Icon className="size-4 shrink-0" aria-hidden />
      {compact ? null : label}
    </Link>
  );
  if (!compact) {
    return link;
  }
  return (
    <Tooltip>
      <TooltipTrigger asChild>{link}</TooltipTrigger>
      <TooltipContent side="right">{label}</TooltipContent>
    </Tooltip>
  );
}

export function NavLinks({ onNavigate, compact = false }: NavLinksProps) {
  const t = useTranslations("nav");
  const pathname = usePathname();
  const modules = useModules();
  const links = modules.data ? moduleLinks(enabledModules(modules.data.data)) : [];
  return (
    <nav className="grid gap-1">
      {MANAGEMENT.map(({ href, label, icon }) => (
        <NavItem key={href} href={href} label={t(label)} icon={icon} pathname={pathname} onNavigate={onNavigate} compact={compact} />
      ))}
      {links.length > 0 ? (
        <div role="group" aria-labelledby="nav-modules" className="mt-4 grid gap-1">
          <p id="nav-modules" className={cn("px-3 pb-1 text-xs font-medium tracking-wide text-sidebar-foreground/60 uppercase", compact && "sr-only")}>
            {t("modulesSection")}
          </p>
          {compact ? <Separator className="mb-1" /> : null}
          {links.map(({ href, label, icon }) => (
            <NavItem key={href} href={href} label={t(label)} icon={icon} pathname={pathname} onNavigate={onNavigate} compact={compact} />
          ))}
        </div>
      ) : null}
    </nav>
  );
}
