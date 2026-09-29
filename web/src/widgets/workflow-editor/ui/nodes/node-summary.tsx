"use client";

import { Fragment } from "react";

import { Tooltip, TooltipContent, TooltipProvider, TooltipTrigger } from "@/shared/ui/primitives";

import type { Part } from "../../model/values-on-nodes";

export function NodeSummary({ parts }: { parts: Part[] }) {
  return (
    <span className="min-w-0 flex-1 truncate font-mono text-[11px] text-muted-foreground">
      {parts.map((part, index) =>
        "template" in part ? (
          <TooltipProvider key={index}>
            <Tooltip>
              <TooltipTrigger asChild>
                <span tabIndex={0} data-template={part.template} className="nodrag rounded bg-primary/12 px-1 font-medium text-foreground outline-none ring-primary/40 focus-visible:ring-2">
                  {part.text}
                </span>
              </TooltipTrigger>
              <TooltipContent className="font-mono text-xs">{part.template}</TooltipContent>
            </Tooltip>
          </TooltipProvider>
        ) : (
          <Fragment key={index}>{part.text}</Fragment>
        ),
      )}
    </span>
  );
}
