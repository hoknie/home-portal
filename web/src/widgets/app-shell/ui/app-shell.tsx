"use client";

import { Menu, PanelLeftClose, PanelLeftOpen } from "lucide-react";
import { useRouter } from "next/navigation";
import { useTranslations } from "next-intl";
import { type ReactNode, useEffect, useState } from "react";

import { EnvironmentSwitch } from "@/features/environment-switch";
import { LanguageSwitch } from "@/features/language-switch";
import { useEnvironment } from "@/entities/environment";
import { useSession } from "@/entities/session";
import { UnauthorizedError, signInLocation } from "@/shared/api";
import { cn } from "@/shared/lib/cn";
import { Button, Separator, Sheet, SheetContent, SheetTitle, SheetTrigger, Skeleton, TooltipProvider } from "@/shared/ui/primitives";

import { readCollapsed, writeCollapsed } from "../model/menu-state";

import { Brand } from "./brand";
import { NavLinks } from "./nav-links";
import { UserMenu } from "./user-menu";

export type AppShellProps = { children: ReactNode; redirectGuests?: boolean };

export function AppShell({ children, redirectGuests = true }: AppShellProps) {
  const t = useTranslations();
  const router = useRouter();
  const session = useSession();
  const environment = useEnvironment();
  const [menuOpen, setMenuOpen] = useState(false);
  const [collapsed, setCollapsed] = useState(readCollapsed);
  const toggle = () => {
    writeCollapsed(!collapsed);
    setCollapsed(!collapsed);
  };
  const signedOut = redirectGuests && session.error instanceof UnauthorizedError;

  useEffect(() => {
    if (signedOut) {
      router.replace(signInLocation(window.location));
    }
  }, [signedOut, router]);

  if (!session.data) {
    return (
      <div className="flex min-h-svh items-center justify-center" aria-busy="true">
        <Skeleton className="h-10 w-48" />
      </div>
    );
  }

  const rail = (
    <div className="flex h-full flex-col items-stretch gap-6 py-5">
      <Brand compact />
      <div className="flex-1 px-2">
        <NavLinks compact />
      </div>
      <Separator />
      <div className="grid justify-items-center gap-2 px-2">
        <UserMenu name={session.data.name} compact />
        <Button type="button" variant="ghost" size="icon" aria-label={t("nav.expandMenu")} title={t("nav.expandMenu")} onClick={toggle}>
          <PanelLeftOpen aria-hidden />
        </Button>
      </div>
    </div>
  );

  const sidebar = (
    <div className="flex min-h-full flex-col gap-6 py-5">
      <Brand />
      <div className="flex-1 px-3">
        <NavLinks onNavigate={() => setMenuOpen(false)} />
      </div>
      <Separator />
      <div className="grid gap-3 px-3">
        <div className="flex flex-wrap items-center justify-between gap-2">
          <EnvironmentSwitch
            environment={environment.data?.environment ?? null}
            detected={environment.data?.detected ?? null}
            switchable={environment.data?.switchable ?? false}
            environments={environment.data?.environments ?? []}
          />
          <LanguageSwitch />
        </div>
        <UserMenu name={session.data.name} />
      </div>
    </div>
  );

  const expanded = (
    <div className="flex h-full flex-col">
      <div className="min-h-0 flex-1 overflow-y-auto" data-menu-scroll>{sidebar}</div>
      <div className="hidden shrink-0 border-t px-3 py-3 md:block">
        <Button type="button" variant="ghost" size="sm" className="w-full justify-start gap-2 text-muted-foreground" onClick={toggle}>
          <PanelLeftClose aria-hidden />
          {t("nav.collapseMenu")}
        </Button>
      </div>
    </div>
  );

  return (
    <div className={cn("min-h-svh md:grid", collapsed ? "md:grid-cols-[4.5rem_1fr]" : "md:grid-cols-[16rem_1fr]")} data-menu={collapsed ? "collapsed" : "expanded"}>
      <TooltipProvider>
        <aside className={cn("glass-panel sticky top-3 m-3 mr-0 hidden h-[calc(100svh-1.5rem)] rounded-xl md:block", collapsed ? "overflow-y-auto" : "overflow-hidden")} aria-label={t("nav.menu")}>
          {collapsed ? rail : expanded}
        </aside>
      </TooltipProvider>
      <div className="flex min-w-0 flex-col">
        <header className="glass-panel sticky top-2 z-30 mx-2 mt-2 flex h-14 items-center gap-3 rounded-xl px-4 md:hidden">
          <Sheet open={menuOpen} onOpenChange={setMenuOpen}>
            <SheetTrigger asChild>
              <Button variant="ghost" size="icon" aria-label={t("nav.openMenu")}>
                <Menu aria-hidden />
              </Button>
            </SheetTrigger>
            <SheetContent side="left" className="w-72 p-0" closeLabel={t("common.close")}>
              <SheetTitle className="sr-only">{t("nav.openMenu")}</SheetTitle>
              {sidebar}
            </SheetContent>
          </Sheet>
          <Brand />
        </header>
        <main className={cn("mx-auto w-full flex-1 px-4 py-6 sm:px-6 lg:px-8 lg:py-10", collapsed ? "max-w-6xl md:max-w-[83.5rem]" : "max-w-6xl")}>{children}</main>
      </div>
    </div>
  );
}
