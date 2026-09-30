"use client";

import { AlertTriangle } from "lucide-react";
import { useTranslations } from "next-intl";

import type { Problem } from "../model/problems";

export type FormProblemsProps = { problems: Problem[] };

export function FormProblems({ problems }: FormProblemsProps) {
  const t = useTranslations();
  if (problems.length === 0) {
    return null;
  }
  const text = (message: string) => (t.has(message as Parameters<typeof t.has>[0]) ? t(message as Parameters<typeof t>[0], { field: "" }) : message);
  return (
    <div role="alert" className="flex items-start gap-3 rounded-xl border border-destructive/30 bg-destructive/5 p-4">
      <AlertTriangle className="mt-0.5 size-5 shrink-0 text-destructive" aria-hidden />
      <div className="grid gap-1">
        <p className="text-sm font-medium">{t("automationBuilder.problems")}</p>
        <ul className="grid gap-0.5 text-sm text-muted-foreground">
          {problems.map((problem) => (
            <li key={`${problem.path}:${problem.message}`}>{t("automationBuilder.problemLine", { path: problem.path, message: text(problem.message) })}</li>
          ))}
        </ul>
      </div>
    </div>
  );
}
