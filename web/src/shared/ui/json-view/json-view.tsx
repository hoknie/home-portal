import { cn } from "@/shared/lib/cn";

export type JsonViewProps = { value: unknown; label?: string; openLevels?: number; className?: string };

const OPEN_OBJECT = "{";
const CLOSE_OBJECT = "}";
const OPEN_LIST = "[";
const CLOSE_LIST = "]";
const COLON = ": ";

function Scalar({ value }: { value: unknown }) {
  const tone =
    typeof value === "string" ? "text-status-up" : typeof value === "number" ? "text-primary" : typeof value === "boolean" ? "text-status-degraded" : "text-muted-foreground";
  return <span className={tone}>{JSON.stringify(value ?? null)}</span>;
}

function Node({ name, value, depth, openLevels }: { name: string | null; value: unknown; depth: number; openLevels: number }) {
  const label = name === null ? null : (
    <span className="text-foreground">
      {name}
      {COLON}
    </span>
  );
  if (value === null || typeof value !== "object") {
    return (
      <li className="break-all">
        {label}
        <Scalar value={value} />
      </li>
    );
  }
  const list = Array.isArray(value);
  const entries: [string, unknown][] = list ? (value as unknown[]).map((item, index) => [String(index), item]) : Object.entries(value as Record<string, unknown>);
  return (
    <li>
      <details open={depth < openLevels} data-depth={depth}>
        <summary className="cursor-pointer select-none">
          {label}
          <span className="text-muted-foreground">
            {list ? OPEN_LIST : OPEN_OBJECT}
            {entries.length}
            {list ? CLOSE_LIST : CLOSE_OBJECT}
          </span>
        </summary>
        <ul className="ms-4 border-s border-glass-edge ps-2">
          {entries.map(([key, item]) => (
            <Node key={key} name={key} value={item} depth={depth + 1} openLevels={openLevels} />
          ))}
        </ul>
      </details>
    </li>
  );
}

export function JsonView({ value, label, openLevels = 1, className }: JsonViewProps) {
  return (
    <ul aria-label={label} className={cn("font-mono text-xs leading-5", className)}>
      <Node name={null} value={value} depth={0} openLevels={openLevels} />
    </ul>
  );
}
