"use client";

import { ChevronDown, type LucideIcon } from "lucide-react";
import Link from "next/link";
import { usePathname } from "next/navigation";
import { useTranslations } from "next-intl";

import { useScripts } from "@/entities/automation";
import { MODULE_NAMES, type ModuleCategory, enabledModules, useModules } from "@/entities/module";
import { mayOpen, useSession } from "@/entities/session";
import { cn } from "@/shared/lib/cn";
import { Separator, Tooltip, TooltipContent, TooltipTrigger } from "@/shared/ui/primitives";

import { toggleCategory, useCollapsedCategories } from "../model/category-state";
import { type CategoryLinks, categoryLinks, isActive, managementLinks } from "../model/navigation";

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

type CategoryProps = { group: CategoryLinks; collapsed: boolean; pathname: string; onNavigate?: () => void };

function Category({ group, collapsed, pathname, onNavigate }: CategoryProps) {
  const t = useTranslations("nav");
  const names = useTranslations("modules.categories");
  const listId = `nav-category-${group.category}`;
  const holdsActive = group.links.some((link) => isActive(pathname, link.href));
  return (
    <div className="grid gap-1" data-category={group.category}>
      <button
        type="button"
        aria-expanded={!collapsed}
        aria-controls={listId}
        data-active={collapsed && holdsActive ? "" : undefined}
        onClick={() => toggleCategory(group.category)}
        className={cn(
          "flex items-center gap-2 rounded-lg px-3 py-1.5 text-left text-xs font-medium text-sidebar-foreground/70 transition-colors hover:bg-sidebar-accent/60",
          collapsed && holdsActive && "bg-sidebar-accent text-sidebar-accent-foreground",
        )}
      >
        <ChevronDown className={cn("size-3.5 shrink-0 transition-transform", collapsed && "-rotate-90")} aria-hidden />
        {names(group.category)}
      </button>
      <div id={listId} role="group" aria-label={names(group.category)} hidden={collapsed} className="grid gap-1">
        {group.links.map(({ href, label, icon }) => (
          <NavItem key={href} href={href} label={t(label)} icon={icon} pathname={pathname} onNavigate={onNavigate} />
        ))}
      </div>
    </div>
  );
}

export function NavLinks({ onNavigate, compact = false }: NavLinksProps) {
  const t = useTranslations("nav");
  const pathname = usePathname();
  const modules = useModules();
  const scripts = useScripts();
  const session = useSession();
  const collapsed = useCollapsedCategories();
  const may = (area: Parameters<typeof mayOpen>[1]) => mayOpen(session.data, area);
  const enabled = modules.data ? enabledModules(modules.data.data) : modules.isError ? new Set(MODULE_NAMES) : new Set<(typeof MODULE_NAMES)[number]>();
  const groups = categoryLinks(enabled, scripts.data?.editing ?? false, may);
  const isCollapsed = (category: ModuleCategory) => collapsed.has(category);
  return (
    <nav className="grid gap-1">
      {managementLinks(may).map(({ href, label, icon }) => (
        <NavItem key={href} href={href} label={t(label)} icon={icon} pathname={pathname} onNavigate={onNavigate} compact={compact} />
      ))}
      {groups.length > 0 ? (
        <div role="group" aria-labelledby="nav-modules" className="mt-4 grid gap-1">
          <p id="nav-modules" className={cn("px-3 pb-1 text-xs font-medium tracking-wide text-sidebar-foreground/60 uppercase", compact && "sr-only")}>
            {t("modulesSection")}
          </p>
          {compact
            ? groups.map((group) => (
                <div key={group.category} className="grid gap-1" data-category={group.category}>
                  <Separator className="mb-1" />
                  {group.links.map(({ href, label, icon }) => (
                    <NavItem key={href} href={href} label={t(label)} icon={icon} pathname={pathname} onNavigate={onNavigate} compact />
                  ))}
                </div>
              ))
            : groups.map((group) => <Category key={group.category} group={group} collapsed={isCollapsed(group.category)} pathname={pathname} onNavigate={onNavigate} />)}
        </div>
      ) : null}
    </nav>
  );
}
