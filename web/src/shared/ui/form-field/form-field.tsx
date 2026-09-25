"use client";

import { useTranslations } from "next-intl";
import type { ReactNode } from "react";

import { Label } from "@/shared/ui/primitives";

export type FormFieldProps = {
  id: string;
  label: string;
  hint?: string;
  error?: string;
  optional?: boolean;
  children: ReactNode;
};

export function FormField({ id, label, hint, error, optional = false, children }: FormFieldProps) {
  const t = useTranslations();
  const message = error && t.has(error as Parameters<typeof t.has>[0]) ? t(error as Parameters<typeof t>[0]) : error;
  return (
    <div className="grid content-start gap-2">
      <Label htmlFor={id} className="flex items-baseline gap-1.5">
        {label}
        {optional ? <span className="text-xs font-normal text-muted-foreground">{t("common.optional")}</span> : null}
      </Label>
      {children}
      {message ? (
        <p id={`${id}-error`} role="alert" className="text-sm text-destructive">
          {message}
        </p>
      ) : hint ? (
        <p id={`${id}-hint`} className="text-xs text-muted-foreground">
          {hint}
        </p>
      ) : null}
    </div>
  );
}
