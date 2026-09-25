"use client";

import { X } from "lucide-react";
import { type KeyboardEvent, useId, useState } from "react";

import { cn } from "@/shared/lib/cn";

export type TagInputProps = {
  id: string;
  values: string[];
  onChange: (values: string[]) => void;
  suggestions: string[];
  max?: number;
  invalid?: boolean;
  describedBy?: string;
  removeLabel: (value: string) => string;
  createLabel: (value: string) => string;
  onBlur?: () => void;
};

type Option = { value: string; created: boolean };

export function optionsFor(text: string, suggestions: string[], chosen: string[]): Option[] {
  const typed = text.trim();
  const needle = typed.toLocaleLowerCase();
  const taken = new Set(chosen.map((value) => value.toLocaleLowerCase()));
  const matching = [...new Set(suggestions)]
    .filter((suggestion) => !taken.has(suggestion.toLocaleLowerCase()))
    .filter((suggestion) => suggestion.toLocaleLowerCase().includes(needle))
    .map((value) => ({ value, created: false }));
  const exact = suggestions.some((suggestion) => suggestion.toLocaleLowerCase() === needle) || taken.has(needle);
  return typed === "" || exact ? matching : [...matching, { value: typed, created: true }];
}

export function TagInput({
  id,
  values,
  onChange,
  suggestions,
  max = Number.POSITIVE_INFINITY,
  invalid = false,
  describedBy,
  removeLabel,
  createLabel,
  onBlur,
}: TagInputProps) {
  const listId = useId();
  const [text, setText] = useState("");
  const [open, setOpen] = useState(false);
  const [active, setActive] = useState(0);
  const options = open ? optionsFor(text, suggestions, values) : [];
  const current = options[Math.min(active, options.length - 1)] ?? null;

  const add = (value: string) => {
    const trimmed = value.trim();
    if (trimmed === "") {
      return;
    }
    const kept = values.filter((existing) => existing.toLocaleLowerCase() !== trimmed.toLocaleLowerCase());
    const next = [...kept, trimmed];
    onChange(next.slice(Math.max(0, next.length - max)));
    setText("");
    setActive(0);
  };

  const remove = (value: string) => onChange(values.filter((existing) => existing !== value));

  const keyDown = (event: KeyboardEvent<HTMLInputElement>) => {
    if (event.key === "ArrowDown" || event.key === "ArrowUp") {
      event.preventDefault();
      setOpen(true);
      const count = optionsFor(text, suggestions, values).length;
      if (count > 0) {
        setActive((index) => (event.key === "ArrowDown" ? (index + 1) % count : (index - 1 + count) % count));
      }
    } else if (event.key === "Enter") {
      event.preventDefault();
      if (current) {
        add(current.value);
      } else {
        add(text);
      }
    } else if (event.key === "Escape") {
      setOpen(false);
    } else if (event.key === "Backspace" && text === "" && values.length > 0) {
      remove(values[values.length - 1]);
    }
  };

  return (
    <div className="relative">
      <div
        className={cn(
          "flex min-h-9 w-full flex-wrap items-center gap-1.5 rounded-md border border-input bg-glass-tint px-2 py-1 shadow-xs",
          "focus-within:border-ring focus-within:ring-[3px] focus-within:ring-ring/50",
          invalid && "border-destructive",
        )}
      >
        {values.map((value) => (
          <span key={value} className="inline-flex items-center gap-1 rounded-full border border-glass-edge bg-glass-tint px-2 py-0.5 text-xs">
            {value}
            <button
              type="button"
              className="rounded-full text-muted-foreground hover:text-foreground focus-visible:ring-2 focus-visible:ring-ring focus-visible:outline-none"
              aria-label={removeLabel(value)}
              onClick={() => remove(value)}
            >
              <X className="size-3" aria-hidden />
            </button>
          </span>
        ))}
        <input
          id={id}
          role="combobox"
          aria-expanded={options.length > 0}
          aria-controls={listId}
          aria-autocomplete="list"
          aria-activedescendant={current ? `${listId}-${options.indexOf(current)}` : undefined}
          aria-invalid={invalid || undefined}
          aria-describedby={describedBy}
          autoComplete="off"
          spellCheck={false}
          className="h-7 min-w-24 flex-1 bg-transparent text-base outline-none md:text-sm"
          value={text}
          onChange={(event) => {
            setText(event.target.value);
            setActive(0);
            setOpen(true);
          }}
          onFocus={() => setOpen(true)}
          onBlur={() => {
            setOpen(false);
            onBlur?.();
          }}
          onKeyDown={keyDown}
        />
      </div>
      {options.length > 0 ? (
        <ul id={listId} role="listbox" className="glass-overlay absolute inset-x-0 top-full z-50 mt-1 max-h-60 overflow-y-auto rounded-lg p-1 text-sm">
          {options.map((option, index) => (
            <li
              key={`${option.created}-${option.value}`}
              id={`${listId}-${index}`}
              role="option"
              aria-selected={option === current}
              className={cn("cursor-pointer rounded-md px-2 py-1.5", option === current && "bg-glass-tint")}
              onMouseDown={(event) => {
                event.preventDefault();
                add(option.value);
              }}
              onMouseEnter={() => setActive(index)}
            >
              {option.created ? createLabel(option.value) : option.value}
            </li>
          ))}
        </ul>
      ) : null}
    </div>
  );
}
