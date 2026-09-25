"use client";

import { House, LogIn, Settings2 } from "lucide-react";
import Link from "next/link";
import { useTranslations } from "next-intl";

import { EnvironmentSwitch, type EnvironmentSwitchProps } from "@/features/environment-switch";
import { LanguageSwitch } from "@/features/language-switch";
import { routes } from "@/shared/config";
import { Button } from "@/shared/ui/primitives";

import { AccountMenu } from "./account-menu";

export type SiteHeaderBarProps = { environment: EnvironmentSwitchProps; user: string | null };

export function SiteHeaderBar({ environment, user }: SiteHeaderBarProps) {
  const t = useTranslations();
  return (
    <header className="glass-panel flex flex-wrap items-center gap-3 rounded-xl px-4 py-3">
      <Link href={routes.home} className="flex items-center gap-2.5">
        <span className="flex size-9 items-center justify-center rounded-lg bg-primary text-primary-foreground shadow-sm">
          <House className="size-4" aria-hidden />
        </span>
        <span className="text-lg font-semibold tracking-tight">{t("common.appName")}</span>
      </Link>
      <EnvironmentSwitch {...environment} />
      <div className="ml-auto flex items-center gap-2">
        <LanguageSwitch />
        {user ? (
          <>
            <Button asChild variant="outline" size="sm">
              <Link href={routes.adminServices}>
                <Settings2 className="size-4" aria-hidden />
                {t("nav.management")}
              </Link>
            </Button>
            <AccountMenu name={user} />
          </>
        ) : (
          <Button asChild variant="outline" size="sm">
            <Link href={routes.login}>
              <LogIn className="size-4" aria-hidden />
              {t("publicPortal.signIn")}
            </Link>
          </Button>
        )}
      </div>
    </header>
  );
}
