"use client";

import { cn } from "cn";
import { CircleAlert, CircleCheck, TriangleAlert } from "lucide-react";
import { useTranslations } from "next-intl";

import { parsePath, pathText } from "@/entities/workflow";

import { useEditor } from "../../model/editor-context";
import { nodeLabel } from "../../model/localize";
import type { Problem } from "../../model/checks/problems";
import { BareButton } from "@/shared/ui/kit";

export type ProblemsPanelProps = { onChoose: (problem: Problem) => void };

export function ProblemsPanel({ onChoose }: ProblemsPanelProps) {
  const t = useTranslations("workflowEditor.problemsPanel");
  const translate = useTranslations();
  const editor = useEditor();
  if (editor.problems.length === 0) {
    return (
      <p className="flex items-center gap-2 p-3 text-sm text-muted-foreground">
        <CircleCheck className="size-4 text-status-up" aria-hidden />
        {t("none")}
      </p>
    );
  }
  return (
    <ul className="grid gap-1 p-2" aria-label={t("title")}>
      {editor.problems.map((problem, index) => {
        const { path, field } = parsePath(problem.at);
        const place = path.length === 0 ? t("workflow") : (nodeLabel(editor.draft.steps, pathText(path)) ?? pathText(path));
        const message = problem.text ?? translate(problem.key as "validation.required", problem.params);
        const Icon = problem.severity === "error" ? CircleAlert : TriangleAlert;
        return (
          <li key={`${problem.at}-${index}`}>
            <BareButton
              onClick={() => onChoose(problem)}
              className="grid w-full grid-cols-[auto_1fr] gap-x-2 rounded-lg p-2 text-start text-sm hover:bg-accent"
            >
              <Icon className={cn("mt-0.5 size-4", problem.severity === "error" ? "text-destructive" : "text-status-degraded")} aria-hidden />
              <span className="grid min-w-0">
                <span className="truncate font-medium">
                  {place}
                  {field ? <span className="ms-1 font-mono text-xs text-muted-foreground">{field}</span> : null}
                </span>
                <span className="text-xs text-muted-foreground">{message}</span>
              </span>
            </BareButton>
          </li>
        );
      })}
    </ul>
  );
}
