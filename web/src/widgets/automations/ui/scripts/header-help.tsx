"use client";

import { BookOpen, ChevronDown, WandSparkles } from "lucide-react";
import { useTranslations } from "next-intl";

import { TOKEN_CLASSES, highlight } from "@/shared/ui/code-area";
import { Button } from "@/shared/ui/primitives";

export const HELP_ROWS = [
  ["description", "# @description Restart a service's container"],
  ["required", "# @arg service <text> Service id"],
  ["optional", "# @arg host? <text> Where to look"],
  ["option", "# @arg --retries <number=3> Tries before giving up"],
  ["flag", "# @arg --force Skip the health check"],
  ["choice", "# @arg mode <fast|full=fast> How deep to check"],
] as const;

export const EXAMPLE_HEADER = HELP_ROWS.filter(([row]) => row !== "optional").map(([, line]) => line);

export function withExampleHeader(text: string) {
  const lines = text.split("\n");
  const at = lines[0]?.startsWith("#!") ? 1 : 0;
  return [...lines.slice(0, at), ...EXAMPLE_HEADER, ...lines.slice(at)].join("\n");
}

function Painted({ line }: { line: string }) {
  return (
    <code className="block overflow-x-auto rounded-md border border-glass-edge bg-glass-tint px-2 py-1 font-mono text-xs whitespace-pre">
      {highlight(line)[0].map((token, index) => (
        <span key={index} className={TOKEN_CLASSES[token.kind]}>
          {token.text}
        </span>
      ))}
    </code>
  );
}

export function HeaderHelp({ onInsert }: { onInsert?: () => void }) {
  const t = useTranslations("scripts.help");
  return (
    <details className="group grid gap-3">
      <summary className="flex cursor-pointer list-none items-center gap-2 text-sm font-medium">
        <BookOpen className="size-4 text-muted-foreground" aria-hidden />
        {t("title")}
        <ChevronDown className="ml-auto size-4 text-muted-foreground transition-transform group-open:rotate-180" aria-hidden />
      </summary>
      <div className="mt-3 grid gap-3 text-sm">
        <p className="text-muted-foreground">{t("intro")}</p>
        <ul className="grid gap-2.5">
          {HELP_ROWS.map(([row, line]) => (
            <li key={row} className="grid gap-1">
              <Painted line={line} />
              <span className="text-xs text-muted-foreground">{t(`rows.${row}`)}</span>
            </li>
          ))}
        </ul>
        <p className="text-xs text-muted-foreground">{t("rules")}</p>
        {onInsert ? (
          <Button type="button" size="sm" variant="outline" className="justify-self-start" onClick={onInsert}>
            <WandSparkles aria-hidden />
            {t("insert")}
          </Button>
        ) : null}
      </div>
    </details>
  );
}
