"use client";

import { useTranslations } from "next-intl";

import { slugOf, uniqueId } from "@/shared/lib/slug";
import { FormField } from "@/shared/ui/form-field";
import { Input, Label, Switch } from "@/shared/ui/primitives";
import { TagInput } from "@/shared/ui/tag-input";

import { useEditor } from "../../model/editor-context";
import { RESERVED_IDS } from "../../model/checks/validation";
import { InputsEditor } from "./header/inputs-editor";
import { OutputsEditor } from "./header/outputs-editor";

export function HeaderForm() {
  const t = useTranslations();
  const editor = useEditor();
  const { draft, header } = editor;
  const problemAt = (at: string) => {
    const found = editor.problems.find((problem) => problem.at === at && problem.severity === "error");
    return found ? (found.text ?? t(found.key as "validation.required", found.params)) : undefined;
  };
  const set = (key: string, change: Partial<typeof draft>) => editor.setDraft((current) => ({ ...current, ...change }), key);
  return (
    <div className="grid gap-4">
      <FormField id="workflow-title" label={t("workflowEditor.title")} error={problemAt("title")}>
        <Input
          id="workflow-title"
          value={draft.title}
          onChange={(change) =>
            set("title", {
              title: change.target.value,
              ...(header.idFollowsTitle ? { id: uniqueId(slugOf(change.target.value), [...header.taken, ...RESERVED_IDS]) } : {}),
            })
          }
        />
      </FormField>
      <div className="grid gap-4 sm:grid-cols-[1fr_auto] sm:items-start">
        <FormField id="workflow-id" label={t("workflowEditor.id")} hint={t("workflowEditor.idHint")} error={problemAt("id")}>
          <Input
            id="workflow-id"
            className="font-mono"
            spellCheck={false}
            autoComplete="off"
            value={draft.id}
            onChange={(change) => {
              set("id", { id: change.target.value });
              header.onIdTyped(change.target.value);
            }}
          />
        </FormField>
        <div className="grid gap-2 sm:pt-0.5">
          <Label htmlFor="workflow-enabled">{t("workflowEditor.enabled")}</Label>
          <Switch id="workflow-enabled" checked={draft.enabled} onCheckedChange={(enabled) => set("enabled", { enabled })} />
        </div>
      </div>
      <FormField id="workflow-description" label={t("workflowEditor.description")} optional>
        <Input id="workflow-description" value={draft.description ?? ""} onChange={(change) => set("description", { description: change.target.value })} />
      </FormField>
      <FormField id="workflow-timeout" label={t("workflowEditor.timeout")} hint={t("workflowEditor.timeoutHint")} error={problemAt("timeout_seconds")}>
        <Input id="workflow-timeout" type="number" inputMode="numeric" value={String(draft.timeout_seconds)} onChange={(change) => set("timeout", { timeout_seconds: Number(change.target.value) })} />
      </FormField>
      <FormField id="workflow-tags" label={t("tags.label")} hint={t("tags.hint")} optional>
        <TagInput
          id="workflow-tags"
          values={draft.tags}
          onChange={(tags) => set("tags", { tags })}
          suggestions={editor.tags}
          removeLabel={(value) => t("tagInput.remove", { value })}
          createLabel={(value) => t("tagInput.create", { value })}
        />
      </FormField>
      <InputsEditor inputs={draft.inputs} problemAt={problemAt} onChange={(key, inputs) => set(key, { inputs })} />
      <OutputsEditor outputs={draft.outputs} problemAt={problemAt} onChange={(key, outputs) => set(key, { outputs })} />
    </div>
  );
}
