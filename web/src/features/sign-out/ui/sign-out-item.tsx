"use client";

import { useQueryClient } from "@tanstack/react-query";
import { LogOut } from "lucide-react";
import { useTranslations } from "next-intl";

import { signOut } from "@/entities/session";
import { routes } from "@/shared/config";
import { DropdownMenuItem } from "@/shared/ui/primitives";

export function SignOutItem() {
  const t = useTranslations("nav");
  const client = useQueryClient();
  const leave = async () => {
    await signOut().catch(() => undefined);
    client.clear();
    window.location.assign(routes.home);
  };
  return (
    <DropdownMenuItem onSelect={leave}>
      <LogOut aria-hidden />
      {t("signOut")}
    </DropdownMenuItem>
  );
}
