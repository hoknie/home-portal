"use client";

import { Menu } from "lucide-react";
import { useRouter } from "next/navigation";
import { useTranslations } from "next-intl";
import { type ReactNode, useEffect, useState } from "react";

import { EnvironmentSwitch } from "@/features/environment-switch";
import { LanguageSwitch } from "@/features/language-switch";
import { useEnvironment } from "@/entities/environment";
import { useSession } from "@/entities/session";
import { UnauthorizedError, signInLocation } from "@/shared/api";
import { Button, Separator, Sheet, SheetContent, SheetTitle, SheetTrigger, Skeleton } from "@/shared/ui/primitives";

import { Brand } from "./brand";
import { NavLinks } from "./nav-links";
import { UserMenu } from "./user-menu";

export type AppShellProps = { children: ReactNode };

export function AppShell({ children }: AppShellProps) {
  const t = useTranslations();
  const router = useRouter();
  const session = useSession();
  const environment = useEnvironment();
  const [menuOpen, setMenuOpen] = useState(false);
  const signedOut = session.error instanceof UnauthorizedError;

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

  const sidebar = (
    <div className="flex h-full flex-col gap-6 py-5">
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

  return (
    <div className="min-h-svh md:grid md:grid-cols-[16rem_1fr]">
      <aside className="glass-panel sticky top-3 m-3 mr-0 hidden h-[calc(100svh-1.5rem)] rounded-xl md:block">{sidebar}</aside>
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
        <main className="mx-auto w-full max-w-6xl flex-1 px-4 py-6 sm:px-6 lg:px-8 lg:py-10">{children}</main>
      </div>
    </div>
  );
}
