"use client";

import { ChevronsUpDown, UserRound } from "lucide-react";
import { useTranslations } from "next-intl";

import { useCan } from "@/entities/session";
import { RestartPortalItem } from "@/features/restart-portal";
import { SignOutItem } from "@/features/sign-out";
import { ThemeSwitch } from "@/features/theme-switch";
import {
  Button,
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuLabel,
  DropdownMenuSeparator,
  DropdownMenuTrigger,
} from "@/shared/ui/kit";

export type UserMenuProps = { name: string; compact?: boolean };

export function UserMenu({ name, compact = false }: UserMenuProps) {
  const t = useTranslations("nav");
  const can = useCan();
  return (
    <DropdownMenu>
      <DropdownMenuTrigger asChild>
        <Button variant="ghost" className={compact ? "mx-auto size-9 p-0" : "w-full justify-start gap-2 px-3"} aria-label={compact ? name : undefined}>
          <span className="flex size-7 shrink-0 items-center justify-center rounded-full border border-glass-edge bg-glass-tint">
            <UserRound className="size-4" aria-hidden />
          </span>
          {compact ? null : <span className="truncate">{name}</span>}
          {compact ? null : <ChevronsUpDown className="ml-auto size-4 opacity-60" aria-hidden />}
        </Button>
      </DropdownMenuTrigger>
      <DropdownMenuContent align="start" className="w-56">
        <DropdownMenuLabel className="font-normal text-muted-foreground">{t("signedInAs", { name })}</DropdownMenuLabel>
        <ThemeSwitch />
        <DropdownMenuSeparator />
        {can("portal", "update") ? <RestartPortalItem /> : null}
        <SignOutItem />
      </DropdownMenuContent>
    </DropdownMenu>
  );
}
