"use client";

import Link from "next/link";
import { useTranslations } from "next-intl";
import { useState } from "react";
import { toast } from "sonner";

import { Allowed } from "@/entities/session";
import { type Module, type ModuleName, type Modules, switchLock, useSwitchModule } from "@/entities/module";
import { RequestError, ValidationError } from "@/shared/api";
import { MODULE_PAGES } from "@/shared/config";
import { Switch } from "@/shared/ui/primitives";

export type ModuleSwitchProps = { module: Module; modules: Modules; revision: string | null };

export function ModuleSwitch({ module, modules, revision }: ModuleSwitchProps) {
  const t = useTranslations("modules");
  const switching = useSwitchModule();
  const [missing, setMissing] = useState<string[]>([]);
  const nameOf = (name: ModuleName) => t(`names.${name}`);
  const listed = (names: ModuleName[]) => names.map(nameOf).join(", ");
  const name = nameOf(module.name);
  const lock = switchLock(module, modules);
  const change = async (enabled: boolean) => {
    setMissing([]);
    try {
      await switching.mutateAsync({ name: module.name, enabled, revision });
      toast.success(t(enabled ? "switchedOn" : "switchedOff", { name }));
    } catch (error) {
      if (error instanceof ValidationError) {
        setMissing(error.fields.map((field) => field.field));
        return;
      }
      const message = error instanceof RequestError && error.message !== "" ? error.message : t("failedUnknown");
      toast.error(t("failed", { name, message }));
    }
  };
  return (
    <div className="grid gap-2">
      <div className="flex items-center gap-3">
        <Allowed area="modules" action="update">
          <Switch
            id={`module-${module.name}`}
            checked={module.enabled}
            disabled={lock !== null || switching.isPending}
            aria-label={t("switchLabel", { name })}
            onCheckedChange={(enabled) => void change(enabled)}
          />
        </Allowed>
        <span className="text-sm font-medium">{t(module.enabled ? "on" : "off")}</span>
      </div>
      {lock ? (
        <p className="text-sm text-muted-foreground">
          {t(lock.kind === "required-by" ? "lockedRequiredBy" : "lockedRequires", { names: listed(lock.modules) })}
        </p>
      ) : null}
      {missing.length > 0 ? (
        <div role="alert" className="grid gap-1 text-sm text-destructive">
          <p>{t("incomplete", { name })}</p>
          <ul className="list-inside list-disc">
            {missing.map((field) => (
              <li key={field}>
                <code>{field}</code>
              </li>
            ))}
          </ul>
          <Link className="underline underline-offset-4" href={MODULE_PAGES[module.name]}>
            {t("configure")}
          </Link>
        </div>
      ) : null}
    </div>
  );
}
