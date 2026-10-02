"use client";

import { useTranslations } from "next-intl";

import { HeaderProblems, type ScriptArgument, type ScriptHeader } from "@/entities/script";
import { cn } from "@/shared/lib/cn";

export const TYPE_CHIPS: Record<ScriptArgument["type"], string> = {
  text: "bg-palette-blue/12 text-palette-blue",
  number: "bg-palette-amber/12 text-palette-amber",
  flag: "bg-palette-violet/12 text-palette-violet",
  choice: "bg-palette-green/12 text-palette-green",
};

export function usageOf(name: string, declared: ScriptArgument[]) {
  const parts = declared.map((argument) => {
    if (argument.option) {
      return argument.type === "flag"
        ? `[${argument.name}]`
        : `[${argument.name} <${argument.type === "choice" ? argument.choices.join("|") : argument.type}>]`;
    }
    return argument.required ? `<${argument.name}>` : `[${argument.name}]`;
  });
  return [name, ...parts].join(" ");
}

function ArgumentCard({ argument }: { argument: ScriptArgument }) {
  const t = useTranslations("scripts");
  return (
    <li className="grid gap-2 rounded-lg border border-glass-edge bg-glass-tint p-3">
      <div className="flex flex-wrap items-center gap-1.5">
        <span className="font-mono text-sm font-semibold">{argument.name}</span>
        <span className={cn("rounded-full px-2 py-0.5 text-xs font-medium", TYPE_CHIPS[argument.type])}>{t(`types.${argument.type}`)}</span>
        <span
          className={cn(
            "rounded-full border px-2 py-0.5 text-xs",
            argument.required ? "border-primary/40 text-primary" : "border-glass-edge text-muted-foreground",
          )}
        >
          {argument.required ? t("required") : t("optional")}
        </span>
      </div>
      {argument.description ? <p className="text-sm">{argument.description}</p> : null}
      {argument.choices.length > 0 || argument.default ? (
        <div className="flex flex-wrap items-center gap-1.5 text-xs text-muted-foreground">
          {argument.choices.map((choice) => (
            <span
              key={choice}
              className={cn("rounded-md border border-glass-edge px-1.5 py-0.5 font-mono", choice === argument.default && "border-primary/40 text-foreground")}
            >
              {choice}
            </span>
          ))}
          {argument.default && argument.choices.length === 0 ? <span className="font-mono">{t("defaultValue", { value: argument.default })}</span> : null}
        </div>
      ) : null}
    </li>
  );
}

export function DeclaredSummary({ name, header }: { name: string; header: ScriptHeader }) {
  const t = useTranslations("scripts");
  return (
    <div className="grid gap-4">
      <div className="grid gap-1">
        <p className="text-xs font-medium tracking-wide text-muted-foreground uppercase">{t("whatItDoes")}</p>
        {header.description ? (
          <p className="text-sm font-medium">{header.description}</p>
        ) : (
          <p className="text-sm text-muted-foreground italic">{t("noDescription")}</p>
        )}
      </div>
      <div className="grid gap-1">
        <p className="text-xs font-medium tracking-wide text-muted-foreground uppercase">{t("usage")}</p>
        <code data-testid="script-usage" className="rounded-md border border-glass-edge bg-glass-tint px-2 py-1.5 font-mono text-xs break-all">
          {usageOf(name, header.arguments)}
        </code>
      </div>
      {header.arguments.length === 0 ? (
        <p className="text-sm text-muted-foreground">{t("noArguments")}</p>
      ) : (
        <ul className="grid gap-2">
          {header.arguments.map((argument) => (
            <ArgumentCard key={argument.name} argument={argument} />
          ))}
        </ul>
      )}
      <HeaderProblems problems={header.problems} />
    </div>
  );
}
