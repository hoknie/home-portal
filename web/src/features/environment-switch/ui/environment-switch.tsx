"use client";

import { useQueryClient } from "@tanstack/react-query";
import { ChevronDown, Undo2 } from "lucide-react";
import { useTranslations } from "next-intl";

import { EnvironmentBadge, clearChoice, iconFor, writeChoice } from "@/entities/environment";
import { cn } from "@/shared/lib/cn";
import {
  Button,
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuLabel,
  DropdownMenuRadioGroup,
  DropdownMenuRadioItem,
  DropdownMenuTrigger,
} from "@/shared/ui/primitives";

export type EnvironmentSwitchProps = {
  environment: string | null;
  detected: string | null;
  switchable: boolean;
  environments: string[];
};

export function EnvironmentSwitch({ environment, detected, switchable, environments }: EnvironmentSwitchProps) {
  const t = useTranslations("environment");
  const client = useQueryClient();
  if (!environment) {
    return null;
  }
  if (!switchable || !detected || environments.length === 0) {
    return <EnvironmentBadge name={environment} />;
  }
  const differs = environment !== detected;
  const choose = (name: string) => {
    if (name === detected) {
      clearChoice();
    } else {
      writeChoice(name);
    }
    void client.invalidateQueries();
  };
  return (
    <div className="flex flex-wrap items-center gap-2">
      <DropdownMenu>
        <DropdownMenuTrigger asChild>
          <button
            type="button"
            data-environment={environment}
            title={t("explains")}
            aria-label={t("choose", { environment })}
            className={cn(
              "inline-flex items-center gap-1.5 rounded-full border px-2.5 py-1 text-xs outline-none focus-visible:ring-2 focus-visible:ring-ring",
              differs ? "border-primary/60 bg-primary/10 text-foreground" : "border-glass-edge bg-glass-tint text-muted-foreground",
            )}
          >
            {iconFor(environment)}
            {environment}
            <ChevronDown className="size-3 opacity-60" aria-hidden />
          </button>
        </DropdownMenuTrigger>
        <DropdownMenuContent align="start" className="w-64">
          <DropdownMenuLabel className="font-normal text-muted-foreground">{t("lookFrom")}</DropdownMenuLabel>
          <DropdownMenuRadioGroup value={environment} onValueChange={choose}>
            {environments.map((name) => (
              <DropdownMenuRadioItem key={name} value={name} className="gap-2">
                {iconFor(name)}
                {name === detected ? t("detectedItem", { name }) : name}
              </DropdownMenuRadioItem>
            ))}
          </DropdownMenuRadioGroup>
        </DropdownMenuContent>
      </DropdownMenu>
      {differs ? (
        <span className="inline-flex items-center gap-1.5 text-xs text-muted-foreground" data-viewing-as={environment}>
          {t("viewingAs", { environment, detected })}
          <Button type="button" variant="ghost" size="sm" className="h-6 gap-1 px-2 text-xs" onClick={() => choose(detected)}>
            <Undo2 className="size-3" aria-hidden />
            {t("back", { detected })}
          </Button>
        </span>
      ) : null}
    </div>
  );
}
