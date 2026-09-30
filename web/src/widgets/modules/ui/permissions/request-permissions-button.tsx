"use client";

import { RefreshCw } from "lucide-react";
import { useTranslations } from "next-intl";

import { useRequestPermissions } from "@/entities/permission";
import { ConflictError, RequestError } from "@/shared/api";
import { Button } from "@/shared/ui/primitives";

export const FORBIDDEN_STATUS = 403;

export function failureKey(error: Error): "busy" | "outside" | "failed" {
  if (error instanceof ConflictError) {
    return "busy";
  }
  if (error instanceof RequestError && error.status === FORBIDDEN_STATUS) {
    return "outside";
  }
  return "failed";
}

export function RequestPermissionsButton() {
  const t = useTranslations("permissions");
  const request = useRequestPermissions();
  return (
    <div className="grid justify-items-start gap-2 sm:justify-items-end">
      <Button onClick={() => request.mutate()} disabled={request.isPending}>
        <RefreshCw aria-hidden />
        {t("requestAgain")}
      </Button>
      {request.error ? (
        <p role="alert" className="max-w-xs text-sm text-destructive">
          {t(`requestErrors.${failureKey(request.error)}`)}
        </p>
      ) : null}
    </div>
  );
}
