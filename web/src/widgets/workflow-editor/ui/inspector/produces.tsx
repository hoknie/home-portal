"use client";

import { Check, Copy } from "lucide-react";
import { useTranslations } from "next-intl";
import { useState } from "react";

import { type Step, sampleOf, typeOfValue } from "@/entities/workflow";

import { useEditor } from "../../model/editor-context";
import { BareButton, Heading } from "@/shared/ui/kit";

export function Produces({ step }: { step: Step }) {
  const t = useTranslations("workflowEditor.produces");
  const help = useTranslations("workflowHelp.results");
  const editor = useEditor();
  const [copied, setCopied] = useState<string | null>(null);
  const types = useTranslations("workflowEditor.transform.types");
  const results = editor.kindOf(step.kind)?.results ?? [];
  const typeOf = (result: string) => {
    if (step.kind !== "transform" || result !== "value") {
      return null;
    }
    const last = sampleOf(`steps.${step.id}.value`, editor.knownAt([], "input"));
    return last === null ? null : typeOfValue(last.value);
  };
  if (results.length === 0) {
    return null;
  }
  const copy = async (reference: string) => {
    try {
      await navigator.clipboard.writeText(reference);
    } catch {
      return;
    }
    setCopied(reference);
  };
  return (
    <section className="grid gap-2" aria-label={t("title")}>
      <div>
        <Heading level="group" as="h3" className="text-sm font-medium">{t("title")}</Heading>
        <p className="text-xs text-muted-foreground">{t("hint")}</p>
      </div>
      <ul className="grid gap-1.5">
        {results.map((result) => {
          const reference = `{{steps.${step.id}.${result}}}`;
          const type = typeOf(result);
          return (
            <li key={result} className="flex min-w-0 items-start gap-2 rounded-lg border border-glass-edge bg-glass-tint p-2">
              <div className="grid min-w-0 flex-1 gap-0.5">
                <span className="flex min-w-0 items-center gap-2">
                  <code className="truncate font-mono text-xs">{reference}</code>
                  {type ? <span className="rounded-sm bg-primary/12 px-1.5 font-mono text-xs text-primary">{types(type)}</span> : null}
                </span>
                <span className="text-xs text-muted-foreground">{help(`${step.kind}.${result}` as "http.status")}</span>
              </div>
              <BareButton
                className="rounded-md p-1 text-muted-foreground hover:bg-accent hover:text-foreground"
                aria-label={t("copy", { reference })}
                onClick={() => void copy(reference)}
              >
                {copied === reference ? <Check className="size-3.5" aria-hidden /> : <Copy className="size-3.5" aria-hidden />}
              </BareButton>
            </li>
          );
        })}
      </ul>
    </section>
  );
}
