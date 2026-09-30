"use client";

import { ShieldOff } from "lucide-react";
import Link from "next/link";
import { useTranslations } from "next-intl";

import { routes } from "@/shared/config";
import { EmptyState } from "@/shared/ui/empty-state";
import { Button } from "@/shared/ui/primitives";

export function NoAccess() {
  const t = useTranslations("access");
  return (
    <div role="status">
      <EmptyState
        icon={ShieldOff}
        title={t("title")}
        description={t("description")}
        action={
          <Button asChild variant="outline" size="sm">
            <Link href={routes.home}>{t("home")}</Link>
          </Button>
        }
      />
    </div>
  );
}
