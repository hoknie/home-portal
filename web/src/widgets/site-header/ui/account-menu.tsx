"use client";

import { ChevronDown, UserRound } from "lucide-react";
import { useTranslations } from "next-intl";

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

export function AccountMenu({ name }: { name: string }) {
  const t = useTranslations("nav");
  return (
    <DropdownMenu>
      <DropdownMenuTrigger asChild>
        <Button variant="ghost" size="sm" className="gap-2">
          <UserRound className="size-4" aria-hidden />
          <span className="max-w-32 truncate">{name}</span>
          <ChevronDown className="size-4 opacity-60" aria-hidden />
        </Button>
      </DropdownMenuTrigger>
      <DropdownMenuContent align="end" className="w-56">
        <DropdownMenuLabel className="font-normal text-muted-foreground">{t("signedInAs", { name })}</DropdownMenuLabel>
        <ThemeSwitch />
        <DropdownMenuSeparator />
        <SignOutItem />
      </DropdownMenuContent>
    </DropdownMenu>
  );
}
