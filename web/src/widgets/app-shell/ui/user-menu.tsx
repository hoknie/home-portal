"use client";

import { ChevronsUpDown, UserRound } from "lucide-react";
import { useTranslations } from "next-intl";

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
} from "@/shared/ui/primitives";

export type UserMenuProps = { name: string };

export function UserMenu({ name }: UserMenuProps) {
  const t = useTranslations("nav");
  return (
    <DropdownMenu>
      <DropdownMenuTrigger asChild>
        <Button variant="ghost" className="w-full justify-start gap-2 px-3">
          <span className="flex size-7 items-center justify-center rounded-full border border-glass-edge bg-glass-tint">
            <UserRound className="size-4" aria-hidden />
          </span>
          <span className="truncate">{name}</span>
          <ChevronsUpDown className="ml-auto size-4 opacity-60" aria-hidden />
        </Button>
      </DropdownMenuTrigger>
      <DropdownMenuContent align="start" className="w-56">
        <DropdownMenuLabel className="font-normal text-muted-foreground">{t("signedInAs", { name })}</DropdownMenuLabel>
        <ThemeSwitch />
        <DropdownMenuSeparator />
        <RestartPortalItem />
        <SignOutItem />
      </DropdownMenuContent>
    </DropdownMenu>
  );
}
