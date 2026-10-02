"use client";

import { useTranslations } from "next-intl";

import { useWorkflowCatalogue } from "@/entities/workflow";
import type { TemplateSuggestion } from "@/shared/ui/kit";

export function useFilterSuggestions(): (subject: string, chain: string) => TemplateSuggestion[] {
  const filters = useWorkflowCatalogue().data?.filters ?? [];
  const help = useTranslations("workflowHelp.filters");
  return () =>
    filters.map((filter) => {
      const required = filter.arguments.filter((argument) => argument.required).map((argument) => argument.name);
      const key = `${filter.name}.description` as "upper.description";
      return {
        value: required.length === 0 ? filter.name : `${filter.name}(${required.join(", ")})`,
        label: filter.arguments.length === 0 ? filter.name : `${filter.name}(${filter.arguments.map((argument) => argument.name).join(", ")})`,
        description: help.has(key) ? help(key) : undefined,
      };
    });
}
