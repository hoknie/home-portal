"use client";

import { useTranslations } from "next-intl";

import type { Scripts } from "../model/schema";
import { fixOf, problemOf, quotedPath } from "../model/script-help";

export function ScriptProblems({ scripts }: { scripts: Scripts }) {
  const t = useTranslations("scriptHelp");
  const broken = scripts.scripts.filter((script) => !script.runnable);
  const directory = quotedPath(scripts.directory);
  const modes = `chmod 755 ${directory} ${directory}/* ${directory}/*/*`;
  return (
    <div className="grid gap-3 text-sm">
      {broken.length > 0 ? (
        <details>
          <summary className="cursor-pointer font-medium">{t("unrunnable", { count: broken.length })}</summary>
          <ul className="mt-2 grid gap-3">
            {broken.map((script) => {
              const problem = problemOf(script.code);
              const fix = problem ? fixOf(problem, script.concerns ?? script.path, scripts.user_id) : null;
              return (
                <li key={script.path} className="grid gap-1" data-problem={problem ?? "unknown"}>
                  <span className="font-mono text-xs">{script.path}</span>
                  <span className="text-muted-foreground">
                    {problem ? t(`problems.${problem}`, { path: script.concerns ?? script.path }) : script.problem}
                  </span>
                  {fix ? <code className="rounded-md border border-glass-edge bg-glass-tint px-2 py-1 font-mono text-xs break-all">{fix}</code> : null}
                </li>
              );
            })}
          </ul>
        </details>
      ) : null}
      <details>
        <summary className="cursor-pointer font-medium">{t("guideTitle")}</summary>
        <ol className="mt-2 grid list-decimal gap-1 pl-5 text-muted-foreground">
          <li>{t("guideFolder", { directory: scripts.directory })}</li>
          <li>{t("guideDepth")}</li>
          <li>{t("guideShebang")}</li>
          <li>
            {t("guideModes")}
            <code className="mt-1 block rounded-md border border-glass-edge bg-glass-tint px-2 py-1 font-mono text-xs break-all">
              {modes}
            </code>
          </li>
          <li>{t("guideOwner", { user: scripts.user_id })}</li>
        </ol>
      </details>
    </div>
  );
}
