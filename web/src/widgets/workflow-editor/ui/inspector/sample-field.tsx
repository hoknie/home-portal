"use client";

import { Braces, History, WandSparkles } from "lucide-react";
import { useTranslations } from "next-intl";
import { useState } from "react";

import { type Path, type Step, jsonKeys, lastOutput, pathText } from "@/entities/workflow";
import { Button, Label } from "@/shared/ui/primitives";

import { useEditor } from "../../model/editor-context";
import { fieldId } from "./template-field";

export const LARGEST_SAMPLE = 16 * 1024;

export function sampleProblem(text: string): "invalid" | "large" | null {
  if (text.trim() === "") {
    return null;
  }
  if (text.length > LARGEST_SAMPLE) {
    return "large";
  }
  try {
    JSON.parse(text);
    return null;
  } catch {
    return "invalid";
  }
}

export function SampleField({ path, step, label, hint }: { path: Path; step: Step; label: string; hint?: string }) {
  const t = useTranslations("workflowEditor.sample");
  const editor = useEditor();
  const [draft, setDraft] = useState(step.response_sample ?? "");
  const id = fieldId(path, "response_sample");
  const answer = lastOutput(editor.lastRun, step.id);
  const fromRun = answer !== null && jsonKeys(answer).length > 0 ? answer : null;
  const problem = sampleProblem(draft);
  const keys = problem === null ? jsonKeys(draft) : [];
  const store = (text: string) => {
    setDraft(text);
    if (sampleProblem(text) === null) {
      editor.change(
        path,
        (current) => {
          const copy = { ...current, response_sample: text.trim() === "" ? undefined : text };
          if (copy.response_sample === undefined) {
            delete copy.response_sample;
          }
          return copy;
        },
        `${pathText(path)}.response_sample`,
      );
    }
  };
  const format = () => {
    try {
      store(JSON.stringify(JSON.parse(draft), null, 2));
    } catch {
      return;
    }
  };
  return (
    <section className="grid gap-2" aria-label={label}>
      <div className="flex items-center gap-2">
        <Braces className="size-4 text-muted-foreground" aria-hidden />
        <Label htmlFor={id}>{label}</Label>
      </div>
      {hint ? <p className="text-xs text-muted-foreground">{hint}</p> : null}
      <div className="flex flex-wrap gap-2">
        <Button type="button" variant="outline" size="sm" disabled={fromRun === null} title={fromRun === null ? t("noRun") : undefined} onClick={() => fromRun && store(JSON.stringify(JSON.parse(fromRun), null, 2))}>
          <History aria-hidden />
          {t("fromRun")}
        </Button>
        <Button type="button" variant="ghost" size="sm" disabled={problem !== null || draft.trim() === ""} onClick={format}>
          <WandSparkles aria-hidden />
          {t("format")}
        </Button>
      </div>
      <textarea
        id={id}
        rows={6}
        spellCheck={false}
        placeholder={t("placeholder")}
        aria-invalid={problem !== null || undefined}
        className="min-h-28 w-full rounded-md border border-input bg-glass-tint px-3 py-2 font-mono text-xs outline-none focus-visible:border-ring focus-visible:ring-[3px] focus-visible:ring-ring/50 aria-invalid:border-destructive"
        value={draft}
        onChange={(change) => store(change.target.value)}
      />
      {problem ? (
        <p role="alert" className="text-sm text-destructive">
          {t(problem)}
        </p>
      ) : null}
      {keys.length > 0 ? (
        <div className="grid gap-1">
          <p className="text-xs text-muted-foreground">{t("keys", { count: keys.length })}</p>
          <ul className="flex flex-wrap gap-1">
            {keys.map((key) => (
              <li key={key.path} data-key={key.path} className="flex max-w-full min-w-0 items-baseline gap-1 rounded-xl border border-glass-edge bg-glass-tint px-2 py-0.5 font-mono text-[11px]">
                <span className="min-w-0 break-all">{key.path}</span>
                <span title={key.example} className="max-w-32 shrink-0 truncate text-muted-foreground">
                  {key.example}
                </span>
              </li>
            ))}
          </ul>
        </div>
      ) : null}
    </section>
  );
}
