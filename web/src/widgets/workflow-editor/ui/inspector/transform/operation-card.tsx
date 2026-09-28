"use client";

import { useSortable } from "@dnd-kit/sortable";
import { CSS } from "@dnd-kit/utilities";
import { ArrowDown, ArrowUp, GripVertical, Trash2 } from "lucide-react";
import { useTranslations } from "next-intl";

import type { Operation, Path, Preview } from "@/entities/workflow";
import { cn } from "@/shared/lib/cn";
import { Button } from "@/shared/ui/primitives";

import { OperationForm } from "./operation-form";
import type { NestedChain } from "./transform-chain";
import { ValuePreview } from "./value-preview";

export type OperationCardProps = {
  id: string;
  path: Path;
  field: string;
  depth: number;
  nested: NestedChain;
  index: number;
  count: number;
  operation: Operation;
  preview: Preview | null;
  from: "sample" | "run" | null;
  item: unknown;
  problem: string | null;
  onChange: (operation: Operation) => void;
  onMove: (offset: -1 | 1) => void;
  onRemove: () => void;
};

export function OperationCard(props: OperationCardProps) {
  const t = useTranslations("workflowEditor.transform");
  const help = useTranslations("workflowHelp.operations");
  const { index, count, operation, preview, problem } = props;
  const { attributes, listeners, setNodeRef, transform, transition, isDragging } = useSortable({ id: props.id });
  const name = help.has(`${operation.op}.name` as "filter.name") ? help(`${operation.op}.name` as "filter.name") : operation.op;
  const title = t("operationTitle", { number: index + 1, name });
  const failed = problem !== null || preview?.error;
  return (
    <li
      ref={setNodeRef}
      style={{ transform: CSS.Translate.toString(transform), transition }}
      aria-label={title}
      data-failed={failed ? "true" : undefined}
      className={cn(
        "grid gap-3 rounded-xl border bg-glass-tint p-3",
        failed ? "border-destructive/60" : "border-glass-edge",
        isDragging && "z-20 opacity-70 shadow-lg",
      )}
    >
      <div className="flex items-center gap-1">
        <button
          type="button"
          className="flex size-7 cursor-grab items-center justify-center rounded-md text-muted-foreground hover:bg-accent focus-visible:ring-2 focus-visible:ring-ring focus-visible:outline-none"
          aria-label={t("drag", { number: index + 1 })}
          {...attributes}
          {...listeners}
        >
          <GripVertical className="size-4" aria-hidden />
        </button>
        <h4 className="min-w-0 flex-1 truncate text-sm font-medium">
          {title}
          {name !== operation.op ? <code className="ml-2 font-mono text-xs text-muted-foreground">{operation.op}</code> : null}
        </h4>
        <Button type="button" variant="ghost" size="icon" className="size-7" aria-label={t("moveUp")} disabled={index === 0} onClick={() => props.onMove(-1)}>
          <ArrowUp aria-hidden />
        </Button>
        <Button type="button" variant="ghost" size="icon" className="size-7" aria-label={t("moveDown")} disabled={index === count - 1} onClick={() => props.onMove(1)}>
          <ArrowDown aria-hidden />
        </Button>
        <Button type="button" variant="ghost" size="icon" className="size-7" aria-label={t("remove", { number: index + 1 })} onClick={props.onRemove}>
          <Trash2 aria-hidden />
        </Button>
      </div>
      {problem ? (
        <p role="alert" className="text-xs text-destructive">
          {problem}
        </p>
      ) : null}
      <OperationForm
        path={props.path}
        field={`${props.field}[${index}]`}
        depth={props.depth}
        operation={operation}
        item={props.item}
        from={props.from}
        nested={props.nested}
        onChange={props.onChange}
      />
      <ValuePreview preview={preview} from={props.from} label={t("after", { number: index + 1 })} />
    </li>
  );
}
