"use client";

import { RadioGroup } from "radix-ui";
import type { ReactNode } from "react";

import { cn } from "@/shared/lib/cn";

import { Tooltip, TooltipContent, TooltipProvider, TooltipTrigger } from "../overlays/tooltip";

export type IconOption<Value extends string> = { value: Value; label: string; icon: ReactNode };

export type IconChoiceProps<Value extends string> = {
  label: string;
  value: Value;
  options: IconOption<Value>[];
  onChange: (value: Value) => void;
  swatch?: boolean;
};

export function IconChoice<Value extends string>({ label, value, options, onChange, swatch = false }: IconChoiceProps<Value>) {
  return (
    <TooltipProvider>
      <fieldset className="grid gap-2">
        <legend className="mb-2 text-sm font-medium">{label}</legend>
        <RadioGroup.Root aria-label={label} value={value} onValueChange={(next) => onChange(next as Value)} orientation="horizontal" loop className="flex flex-wrap gap-1.5">
          {options.map((option) => (
            <Tooltip key={option.value}>
              <TooltipTrigger asChild>
                <RadioGroup.Item
                  value={option.value}
                  aria-label={option.label}
                  data-icon-choice=""
                  className={cn(
                    "inline-flex items-center justify-center border text-muted-foreground transition-colors outline-none hover:bg-muted focus-visible:ring-[3px] focus-visible:ring-ring/50 aria-checked:border-primary aria-checked:bg-primary/10 aria-checked:text-foreground [&_svg]:size-4",
                    swatch ? "size-8 rounded-full p-1 aria-checked:ring-2 aria-checked:ring-primary" : "size-9 rounded-md border-border",
                  )}
                >
                  {option.icon}
                </RadioGroup.Item>
              </TooltipTrigger>
              <TooltipContent>{option.label}</TooltipContent>
            </Tooltip>
          ))}
        </RadioGroup.Root>
      </fieldset>
    </TooltipProvider>
  );
}
