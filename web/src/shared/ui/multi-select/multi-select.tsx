"use client";

import { ChevronDown } from "lucide-react";

import { cn } from "@/shared/lib/cn";

import { DropdownMenu, DropdownMenuCheckboxItem, DropdownMenuContent, DropdownMenuTrigger } from "../primitives";

export type MultiSelectOption = { value: string; label: string; disabled?: boolean };

export type MultiSelectProps = {
  id?: string;
  label: string;
  options: MultiSelectOption[];
  values: string[];
  placeholder: string;
  disabled?: boolean;
  invalid?: boolean;
  onChange: (values: string[]) => void;
};

export function MultiSelect({ id, label, options, values, placeholder, disabled = false, invalid = false, onChange }: MultiSelectProps) {
  const chosen = options.filter((option) => values.includes(option.value));
  const toggle = (value: string, on: boolean) => onChange(options.map((option) => option.value).filter((name) => (name === value ? on : values.includes(name))));
  return (
    <DropdownMenu>
      <DropdownMenuTrigger
        id={id}
        disabled={disabled}
        aria-label={label}
        aria-invalid={invalid ? true : undefined}
        className={cn(
          "flex h-9 w-full min-w-0 items-center justify-between gap-2 rounded-md border border-input bg-glass-tint px-3 text-left text-sm outline-none focus-visible:border-ring focus-visible:ring-[3px] focus-visible:ring-ring/50 disabled:opacity-50 aria-invalid:border-destructive",
        )}
      >
        <span className={cn("truncate", chosen.length === 0 && "text-muted-foreground")}>{chosen.length === 0 ? placeholder : chosen.map((option) => option.label).join(", ")}</span>
        <ChevronDown className="size-4 shrink-0 opacity-60" aria-hidden />
      </DropdownMenuTrigger>
      <DropdownMenuContent align="start" className="min-w-(--radix-dropdown-menu-trigger-width)">
        {options.map((option) => (
          <DropdownMenuCheckboxItem
            key={option.value}
            checked={values.includes(option.value)}
            disabled={option.disabled}
            onSelect={(event) => event.preventDefault()}
            onCheckedChange={(on) => toggle(option.value, on === true)}
          >
            {option.label}
          </DropdownMenuCheckboxItem>
        ))}
      </DropdownMenuContent>
    </DropdownMenu>
  );
}
