"use client";

import { ChevronDown, Languages } from "lucide-react";
import { useLocale, useTranslations } from "next-intl";

import { isLocale, locales } from "@/shared/i18n";
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuLabel,
  DropdownMenuRadioGroup,
  DropdownMenuRadioItem,
  DropdownMenuTrigger,
} from "@/shared/ui/primitives";

import { chooseLanguage } from "../model/choose";

export function LanguageSwitch() {
  const t = useTranslations("language");
  const current = useLocale();
  const choose = (value: string) => {
    if (isLocale(value) && value !== current) {
      chooseLanguage(value);
    }
  };
  return (
    <DropdownMenu>
      <DropdownMenuTrigger asChild>
        <button
          type="button"
          aria-label={t("choose", { name: t(`names.${current}`) })}
          className="inline-flex items-center gap-1.5 rounded-full border border-glass-edge bg-glass-tint px-2.5 py-1 text-xs text-muted-foreground uppercase outline-none focus-visible:ring-2 focus-visible:ring-ring"
        >
          <Languages className="size-3.5" aria-hidden />
          {current}
          <ChevronDown className="size-3 opacity-60" aria-hidden />
        </button>
      </DropdownMenuTrigger>
      <DropdownMenuContent align="end" className="w-44">
        <DropdownMenuLabel className="font-normal text-muted-foreground">{t("label")}</DropdownMenuLabel>
        <DropdownMenuRadioGroup value={current} onValueChange={choose}>
          {locales.map((locale) => (
            <DropdownMenuRadioItem key={locale} value={locale} lang={locale}>
              {t(`names.${locale}`)}
            </DropdownMenuRadioItem>
          ))}
        </DropdownMenuRadioGroup>
      </DropdownMenuContent>
    </DropdownMenu>
  );
}
