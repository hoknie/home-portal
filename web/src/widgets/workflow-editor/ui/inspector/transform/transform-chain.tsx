"use client";

import { DndContext, type DragEndEvent, KeyboardSensor, PointerSensor, closestCenter, useSensor, useSensors } from "@dnd-kit/core";
import { SortableContext, arrayMove, sortableKeyboardCoordinates, verticalListSortingStrategy } from "@dnd-kit/sortable";
import { Plus } from "lucide-react";
import { useTranslations } from "next-intl";
import { type ReactNode, useState } from "react";

import {
  DEEPEST_EACH,
  LIST_OPERATIONS,
  MOST_OPERATIONS,
  type Operation,
  type Path,
  type Sample,
  type Step,
  emptyRow,
  inputSample,
  pathText,
  previewChain,
  settled,
  typeOfValue,
} from "@/entities/workflow";
import { Button, Heading, NativeSelect } from "@/shared/ui/kit";

import { useEditor } from "../../../model/editor-context";
import { OperationCard } from "./operation-card";

export const FILTER_GROUPS = ["list", "text", "number", "object", "any"] as const;

function fresh(op: string): Operation {
  switch (op) {
    case "filter":
      return { op, where: emptyRow() };
    case "map":
      return { op, to: "{{item}}" };
    case "each":
      return { op, operations: [] };
    case "sort_by":
    case "group_by":
    case "count_by":
      return { op, key: "" };
    default:
      return { op };
  }
}

export type NestedChain = (props: OperationsListProps) => ReactNode;

export type OperationsListProps = {
  path: Path;
  field: string;
  operations: Operation[];
  input: Sample | null;
  depth: number;
  onChange: (operations: Operation[]) => void;
};

export function OperationsList({ path, field, operations, input, depth, onChange }: OperationsListProps) {
  const t = useTranslations("workflowEditor.transform");
  const help = useTranslations("workflowHelp.operations");
  const filterHelp = useTranslations("workflowHelp.filters");
  const editor = useEditor();
  const [chosen, setChosen] = useState<string | null>(null);
  const sensors = useSensors(useSensor(PointerSensor, { activationConstraint: { distance: 4 } }), useSensor(KeyboardSensor, { coordinateGetter: sortableKeyboardCoordinates }));
  const ids = operations.map((_, index) => `${field}-${index}`);
  const at = pathText(path);
  const known = editor.knownAt(path, field);
  const previews = input === null ? null : previewChain(input.value, operations, known).steps;
  const filters = editor.catalogue.filters;
  const offered = LIST_OPERATIONS.filter((name) => name !== "each" || depth < DEEPEST_EACH);
  const labelOf = (name: string) => (filterHelp.has(`${name}.label` as "get.label") ? filterHelp(`${name}.label` as "get.label") : name);
  const drop = (event: DragEndEvent) => {
    const from = ids.indexOf(String(event.active.id));
    const to = event.over ? ids.indexOf(String(event.over.id)) : -1;
    if (from >= 0 && to >= 0 && from !== to) {
      onChange(arrayMove(operations, from, to));
    }
  };
  const problemAt = (index: number) => {
    const prefix = `${at}.${field}[${index}]`;
    const found = editor.problems.find((problem) => problem.at === prefix || problem.at?.startsWith(`${prefix}.`));
    return found ? (found.text ?? found.key) : null;
  };
  const final = previews?.at(-1);
  const last = operations.length === 0 ? (input === null ? undefined : input.value) : settled(final) ? final.value : undefined;
  const now = last === undefined || last === null ? null : typeOfValue(last);
  const fitting =
    now === null
      ? []
      : [
          ...(now === "list" ? offered.map((name) => ({ name, label: help(`${name}.name`) })) : []),
          ...filters
            .filter((filter) => filter.accepts.includes(now) || filter.accepts.includes("any") || (now === "list" && filter.element))
            .map((filter) => ({
              name: filter.name,
              label: now === "list" && filter.element && !filter.accepts.includes("list") ? t("forEachElement", { name: filter.name }) : labelOf(filter.name),
            })),
        ];
  const choice = chosen ?? fitting[0]?.name ?? LIST_OPERATIONS[0];
  const setChoice = setChosen;
  const before = (index: number) => {
    const previous = previews?.[index - 1];
    return index === 0 ? (input?.value ?? null) : settled(previous) ? previous.value : null;
  };
  return (
    <div className="grid gap-2">
      {operations.length === 0 ? <p className="text-sm text-muted-foreground">{t("empty")}</p> : null}
      <DndContext sensors={sensors} collisionDetection={closestCenter} onDragEnd={drop}>
        <SortableContext items={ids} strategy={verticalListSortingStrategy}>
          <ol className="grid gap-2" aria-label={depth === 0 ? t("chain") : t("nestedChain", { depth })}>
            {operations.map((operation, index) => {
              const value = before(index);
              return (
                <OperationCard
                  key={ids[index]}
                  id={ids[index]}
                  path={path}
                  field={field}
                  depth={depth}
                  index={index}
                  count={operations.length}
                  operation={operation}
                  preview={previews ? previews[index] : null}
                  from={input?.from ?? null}
                  item={Array.isArray(value) ? value[0] : value !== null && typeof value === "object" ? value : undefined}
                  problem={problemAt(index)}
                  nested={OperationsList}
                  onChange={(next) => onChange(operations.map((current, position) => (position === index ? next : current)))}
                  onMove={(offset) => onChange(arrayMove(operations, index, index + offset))}
                  onRemove={() => onChange(operations.filter((_, position) => position !== index))}
                />
              );
            })}
          </ol>
        </SortableContext>
      </DndContext>
      <div className="flex flex-wrap items-center gap-2">
        {now ? <p className="w-full text-xs text-muted-foreground">{t("valueNow", { type: t(`types.${now}`) })}</p> : null}
        <NativeSelect aria-label={t("choose")} className="w-auto min-w-48 flex-1" value={choice} onChange={(event) => setChoice(event.target.value)}>
          {fitting.length > 0 ? (
            <optgroup label={t("groups.fitting", { type: t(`types.${now ?? "any"}`) })}>
              {fitting.map((option) => (
                <option key={`fitting-${option.name}`} value={option.name}>
                  {option.label}
                </option>
              ))}
            </optgroup>
          ) : null}
          <optgroup label={t("groups.operations")}>
            {offered.map((name) => (
              <option key={name} value={name}>
                {help(`${name}.name`)}
              </option>
            ))}
          </optgroup>
          {FILTER_GROUPS.map((group) => (
            <optgroup key={group} label={t(`groups.${group}`)}>
              {filters
                .filter((filter) => filter.accepts.includes(group))
                .map((filter) => (
                  <option key={`${group}-${filter.name}`} value={filter.name}>
                    {labelOf(filter.name)}
                  </option>
                ))}
            </optgroup>
          ))}
        </NativeSelect>
        <Button type="button" variant="outline" size="sm" disabled={operations.length >= MOST_OPERATIONS} onClick={() => onChange([...operations, fresh(choice)])}>
          <Plus aria-hidden />
          {t("add")}
        </Button>
      </div>
    </div>
  );
}

export function TransformChain({ path, step, label, hint }: { path: Path; step: Step; label: string; hint?: string }) {
  const editor = useEditor();
  const at = pathText(path);
  const input = inputSample(step, editor.knownAt(path, "operations"));
  return (
    <section className="grid gap-3" aria-label={label}>
      <div>
        <Heading level="group" as="h3" className="text-sm font-medium">{label}</Heading>
        {hint ? <p className="text-xs text-muted-foreground">{hint}</p> : null}
      </div>
      <OperationsList
        path={path}
        field="operations"
        operations={step.operations ?? []}
        input={input}
        depth={0}
        onChange={(next) => editor.change(path, (current) => ({ ...current, operations: next }), `${at}.operations`)}
      />
    </section>
  );
}
