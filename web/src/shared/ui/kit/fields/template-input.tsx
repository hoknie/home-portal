"use client";

import { cn } from "cn";
import { useTranslations } from "next-intl";
import { type KeyboardEvent, useId, useLayoutEffect, useRef, useState } from "react";

import {
  type Completion,
  type Mode,
  type TemplateRange,
  type TemplateSuggestion,
  type Trigger,
  applied,
  argumentCompletionAt,
  completionAt,
  filterCompletionAt,
  matching,
  segmentsOf,
} from "./completion";

export type TemplateInputProps = {
  id?: string;
  value: string;
  onChange: (value: string) => void;
  suggestions: TemplateSuggestion[];
  templateSuggestions?: TemplateSuggestion[];
  filterSuggestions?: (subject: string, chain: string) => TemplateSuggestion[];
  trigger?: Trigger;
  multiline?: boolean;
  problems?: TemplateRange[];
  placeholder?: string;
  className?: string;
  groupLabel?: (group: string) => string;
  onBlur?: () => void;
  onCaret?: (caret: number | null) => void;
  "aria-label"?: string;
  "aria-invalid"?: boolean;
  "aria-describedby"?: string;
};

const FRAME =
  "relative w-full min-w-0 rounded-md border border-input bg-glass-tint transition-[color,box-shadow] focus-within:border-ring focus-within:ring-[3px] focus-within:ring-ring/50 has-[[aria-invalid=true]]:border-destructive";
const TEXT = "w-full px-3 py-2 font-mono text-sm leading-5 whitespace-pre-wrap break-words";

type Field = HTMLInputElement | HTMLTextAreaElement;

function grouped(items: TemplateSuggestion[]) {
  const groups: { group: string; items: { suggestion: TemplateSuggestion; index: number }[] }[] = [];
  items.forEach((suggestion, index) => {
    const group = suggestion.group ?? "";
    const last = groups.at(-1);
    if (last && last.group === group) {
      last.items.push({ suggestion, index });
    } else {
      groups.push({ group, items: [{ suggestion, index }] });
    }
  });
  return groups;
}

export function TemplateInput({
  id,
  value,
  onChange,
  suggestions,
  templateSuggestions,
  filterSuggestions,
  trigger = "braces",
  multiline = false,
  problems = [],
  placeholder,
  className,
  groupLabel = (group) => group,
  onBlur,
  onCaret,
  ...aria
}: TemplateInputProps) {
  const t = useTranslations("templateInput");
  const fallbackId = useId();
  const fieldId = id ?? fallbackId;
  const listId = `${fieldId}-suggestions`;
  const field = useRef<Field>(null);
  const mirror = useRef<HTMLDivElement>(null);
  const pendingCaret = useRef<number | null>(null);
  const [completion, setCompletion] = useState<Completion | null>(null);
  const [mode, setMode] = useState<Mode>(trigger);
  const [active, setActive] = useState(0);
  const pool =
    mode === "filters" && completion && filterSuggestions
      ? filterSuggestions(completion.subject ?? "", completion.chain ?? "")
      : mode === "arguments"
        ? trigger === "braces"
          ? suggestions
          : (templateSuggestions ?? [])
        : mode === trigger
        ? suggestions
        : (templateSuggestions ?? []);
  const shown = completion ? matching(pool, completion.query) : [];
  const open = completion !== null && (shown.length > 0 || mode === "braces");

  useLayoutEffect(() => {
    const element = field.current;
    if (element && pendingCaret.current !== null) {
      element.setSelectionRange(pendingCaret.current, pendingCaret.current);
      pendingCaret.current = null;
    }
  });

  const sync = () => {
    const element = field.current;
    onCaret?.(element?.selectionStart ?? null);
    if (element && mirror.current) {
      mirror.current.scrollLeft = element.scrollLeft;
      mirror.current.scrollTop = element.scrollTop;
    }
  };

  const refresh = (text: string, caret: number | null) => {
    const argument = caret === null || !filterSuggestions ? null : argumentCompletionAt(text, caret);
    if (argument) {
      setMode("arguments");
      setCompletion(argument);
      setActive(0);
      return;
    }
    const filters = caret === null || !filterSuggestions ? null : filterCompletionAt(text, caret);
    if (filters) {
      setMode("filters");
      setCompletion(filters);
      setActive(0);
      return;
    }
    const braces = caret === null || trigger === "braces" || !templateSuggestions ? null : completionAt(text, caret, "braces");
    const next = braces ?? (caret === null ? null : completionAt(text, caret, trigger));
    setMode(braces ? "braces" : trigger);
    setCompletion(next);
    setActive(0);
  };

  const choose = (suggestion: TemplateSuggestion) => {
    if (!completion || suggestion.disabled) {
      return;
    }
    const result = applied(value, completion, suggestion, mode);
    pendingCaret.current = result.caret;
    onChange(result.value);
    setCompletion(null);
    field.current?.focus();
  };

  const keyDown = (event: KeyboardEvent<Field>) => {
    if (event.key === " " && (event.ctrlKey || event.metaKey)) {
      event.preventDefault();
      const caret = event.currentTarget.selectionStart ?? value.length;
      if (trigger === "braces" && completionAt(value, caret, trigger) === null) {
        const next = `${value.slice(0, caret)}{{${value.slice(caret)}`;
        pendingCaret.current = caret + 2;
        onChange(next);
        setCompletion({ start: caret + 2, end: caret + 2, query: "" });
      } else {
        refresh(value, caret);
      }
      return;
    }
    if (!open) {
      return;
    }
    if (event.key === "ArrowDown" || event.key === "ArrowUp") {
      event.preventDefault();
      const step = event.key === "ArrowDown" ? 1 : -1;
      setActive((current) => (shown.length === 0 ? 0 : (current + step + shown.length) % shown.length));
    } else if ((event.key === "Enter" || event.key === "Tab") && shown[active]) {
      event.preventDefault();
      choose(shown[active]);
    } else if (event.key === "Escape") {
      event.preventDefault();
      setCompletion(null);
    }
  };

  const common = {
    id: fieldId,
    ref: field as never,
    value,
    placeholder,
    role: "combobox",
    spellCheck: false,
    autoComplete: "off",
    "aria-autocomplete": "list" as const,
    "aria-expanded": open,
    "aria-controls": open ? listId : undefined,
    "aria-activedescendant": open && shown[active] ? `${listId}-${active}` : undefined,
    ...aria,
    className: cn(TEXT, "absolute inset-0 resize-none bg-transparent text-transparent caret-foreground outline-none selection:bg-primary/30 placeholder:text-muted-foreground"),
    onChange: (event: { target: Field }) => {
      onChange(event.target.value);
      refresh(event.target.value, event.target.selectionStart);
    },
    onKeyDown: keyDown,
    onScroll: sync,
    onSelect: sync,
    onClick: (event: { currentTarget: Field }) => {
      sync();
      refresh(value, event.currentTarget.selectionStart);
    },
    onFocus: sync,
    onKeyUp: sync,
    onBlur: () => {
      setCompletion(null);
      onBlur?.();
    },
  };

  return (
    <div className={cn(FRAME, className)}>
      <div ref={mirror} aria-hidden className={cn(TEXT, "pointer-events-none overflow-hidden text-foreground", multiline ? "min-h-20" : "h-9 whitespace-pre")}>
        {segmentsOf(value, problems).map((segment) => (
          <span
            key={segment.start}
            data-template={segment.template || undefined}
            data-problem={segment.problem ? (segment.problem.severity ?? "error") : undefined}
            className={cn(
              segment.template && "rounded-sm bg-primary/15 text-primary",
              segment.problem && "underline decoration-wavy underline-offset-4",
              segment.problem?.severity === "warning" ? "decoration-status-degraded" : segment.problem && "decoration-destructive",
            )}
          >
            {segment.text}
          </span>
        ))}
        {value === "" ? "​" : null}
      </div>
      {multiline ? <textarea {...common} rows={3} /> : <input {...common} type="text" />}
      {open ? (
        <div
          id={listId}
          role="listbox"
          aria-label={t("suggestions")}
          className="absolute top-full left-0 z-50 mt-1 max-h-72 w-full min-w-64 overflow-y-auto rounded-lg border border-glass-edge bg-[var(--glass-overlay-solid)] text-popover-foreground shadow-[var(--glass-shadow)] p-1 text-sm"
        >
          {shown.length === 0 ? <p className="px-2 py-1.5 text-xs text-muted-foreground">{t("noMatches")}</p> : null}
          {grouped(shown).map((group) => (
            <div key={group.group} role="group" aria-label={group.group === "" ? undefined : groupLabel(group.group)}>
              {group.group === "" ? null : <p className="px-2 pt-2 pb-1 text-xs font-medium text-muted-foreground">{groupLabel(group.group)}</p>}
              {group.items.map(({ suggestion, index }) => (
                <div
                  key={suggestion.value}
                  id={`${listId}-${index}`}
                  role="option"
                  aria-selected={index === active}
                  aria-disabled={suggestion.disabled || undefined}
                  onMouseDown={(event) => event.preventDefault()}
                  onMouseEnter={() => setActive(index)}
                  onClick={() => choose(suggestion)}
                  className={cn(
                    "grid min-w-0 cursor-pointer gap-0.5 rounded-md px-2 py-1.5",
                    index === active && "bg-accent",
                    suggestion.disabled && "cursor-not-allowed opacity-50",
                  )}
                >
                  <span data-part="value" className="font-mono text-xs break-all">
                    {suggestion.label ?? suggestion.value}
                  </span>
                  {suggestion.example ? (
                    <span data-part="example" title={suggestion.example} className="truncate font-mono text-xs text-muted-foreground">
                      {suggestion.example}
                    </span>
                  ) : null}
                  {suggestion.description ? <span className="text-xs text-muted-foreground">{suggestion.description}</span> : null}
                  {suggestion.warning ? <span className="text-xs text-status-degraded">{suggestion.warning}</span> : null}
                </div>
              ))}
            </div>
          ))}
        </div>
      ) : null}
    </div>
  );
}
