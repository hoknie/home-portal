"use client";

import { Copy, CornerDownRight, EllipsisVertical, Trash2 } from "lucide-react";
import { useTranslations } from "next-intl";

import { type Path, type Step, pathText } from "@/entities/workflow";
import {
  Button,
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuSeparator,
  DropdownMenuSub,
  DropdownMenuSubContent,
  DropdownMenuSubTrigger,
  DropdownMenuTrigger,
} from "@/shared/ui/primitives";

import { type Destination, destinationsFor } from "../../model/edits/destinations";
import { useEditor } from "../../model/editor-context";

function useDestinationLabel() {
  const t = useTranslations("workflowEditor.destinations");
  return (destination: Destination) => {
    if (destination.owner === null) {
      return t("top");
    }
    const step = destination.owner.label ?? destination.owner.id;
    const branch = /^branches\[(\d+)\]$/.exec(destination.list);
    if (branch) {
      return t("branch", { step, number: Number(branch[1]) + 1 });
    }
    return t(destination.list as "body", { step });
  };
}

export function NodeMenu({ path, step }: { path: Path; step: Step }) {
  const t = useTranslations("workflowEditor");
  const editor = useEditor();
  const label = useDestinationLabel();
  const destinations = destinationsFor(editor.draft.steps, path);
  return (
    <DropdownMenu>
      <DropdownMenuTrigger asChild>
        <Button type="button" variant="ghost" size="icon" className="nodrag size-7" aria-label={t("blockMenu", { id: step.id })} onClick={(event) => event.stopPropagation()}>
          <EllipsisVertical aria-hidden />
        </Button>
      </DropdownMenuTrigger>
      <DropdownMenuContent align="end">
        <DropdownMenuSub>
          <DropdownMenuSubTrigger disabled={destinations.length === 0}>
            <CornerDownRight aria-hidden />
            {t("moveInto")}
          </DropdownMenuSubTrigger>
          <DropdownMenuSubContent className="max-h-72 overflow-y-auto">
            {destinations.map((destination) => (
              <DropdownMenuItem key={`${pathText(destination.target.owner)}-${destination.list}`} onSelect={() => editor.move(path, destination.target)}>
                {label(destination)}
              </DropdownMenuItem>
            ))}
          </DropdownMenuSubContent>
        </DropdownMenuSub>
        <DropdownMenuSeparator />
        <DropdownMenuItem onSelect={() => editor.duplicate(path)}>
          <Copy aria-hidden />
          {t("duplicate")}
        </DropdownMenuItem>
        <DropdownMenuItem variant="destructive" onSelect={() => editor.remove(path)}>
          <Trash2 aria-hidden />
          {t("delete")}
        </DropdownMenuItem>
      </DropdownMenuContent>
    </DropdownMenu>
  );
}
