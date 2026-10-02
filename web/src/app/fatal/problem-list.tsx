import type { Problem } from "@/entities/portal";

export function ProblemList({ problems }: { problems: Problem[] }) {
  return (
    <ul className="grid gap-3">
      {problems.map((problem, index) => (
        <li key={`${problem.file ?? ""}:${problem.field ?? ""}:${index}`} className="grid gap-1 rounded-xl border p-3">
          {problem.file ? <span className="font-mono text-xs break-all text-muted-foreground">{problem.file}</span> : null}
          {problem.field ? <code className="font-mono text-sm font-semibold break-all">{problem.field}</code> : null}
          <span className="text-sm break-words whitespace-pre-wrap">{problem.message}</span>
        </li>
      ))}
    </ul>
  );
}
