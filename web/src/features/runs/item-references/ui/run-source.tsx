"use client";

import type { Run } from "@/entities/automation";
import { ItemReference } from "@/shared/ui/item-reference";

import type { ItemReferences } from "../model/use-item-references";

export function RunSource({ run, references }: { run: Run; references: ItemReferences }) {
  const { reference, also } = references.sourceOf(run);
  return <ItemReference {...reference} also={also} />;
}
