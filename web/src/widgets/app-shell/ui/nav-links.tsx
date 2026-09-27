"use client";

import { House, type LucideIcon } from "lucide-react";
import Link from "next/link";
import { usePathname } from "next/navigation";
import { useTranslations } from "next-intl";

import { enabledModules, useModules } from "@/entities/module";
import { routes } from "@/shared/config";
import { cn } from "@/shared/lib/cn";

import { MANAGEMENT, isActive, moduleLinks } from "../model/navigation";

export type NavLinksProps = { onNavigate?: () => void };

type NavItemProps = { href: string; label: string; icon: LucideIcon; pathname: string; onNavigate?: () => void };

function NavItem({ href, label, icon: Icon, pathname, onNavigate }: NavItemProps) {
  const active = isActive(pathname, href);
  return (
    <Link
      href={href}
      onClick={onNavigate}
      aria-current={active ? "page" : undefined}
      className={cn(
        "flex items-center gap-3 rounded-lg px-3 py-2 text-sm font-medium transition-colors",
        active ? "bg-sidebar-accent text-sidebar-accent-foreground" : "text-sidebar-foreground/80 hover:bg-sidebar-accent/60",
      )}
    >
      <Icon className="size-4" aria-hidden />
      {label}
    </Link>
  );
}

export function NavLinks({ onNavigate }: NavLinksProps) {
  const t = useTranslations("nav");
  const pathname = usePathname();
  const modules = useModules();
  const links = modules.data ? moduleLinks(enabledModules(modules.data.data)) : [];
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
      {MANAGEMENT.map(({ href, label, icon }) => (
        <NavItem key={href} href={href} label={t(label)} icon={icon} pathname={pathname} onNavigate={onNavigate} />
      ))}
      {links.length > 0 ? (
        <div role="group" aria-labelledby="nav-modules" className="mt-4 grid gap-1">
          <p id="nav-modules" className="px-3 pb-1 text-xs font-medium tracking-wide text-sidebar-foreground/60 uppercase">
            {t("modulesSection")}
          </p>
          {links.map(({ href, label, icon }) => (
            <NavItem key={href} href={href} label={t(label)} icon={icon} pathname={pathname} onNavigate={onNavigate} />
          ))}
        </div>
      ) : null}
    </nav>
  );
}
