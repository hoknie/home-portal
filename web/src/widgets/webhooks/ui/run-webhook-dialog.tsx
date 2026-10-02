"use client";

import { Play } from "lucide-react";
import { useTranslations } from "next-intl";
import { useState } from "react";

import { type Webhook, useRunWebhook } from "@/entities/webhook";
import { RequestError, ValidationError } from "@/shared/api";
import { Button, Dialog, DialogContent, DialogDescription, DialogFooter, DialogHeader, DialogTitle, FormField, Input, Textarea } from "@/shared/ui/kit";


export type RunWebhookDialogProps = { webhook: Webhook };

function bodyOf(text: string): { value?: unknown; invalid: boolean } {
  if (text.trim() === "") {
    return { invalid: false };
  }
  try {
    return { value: JSON.parse(text) as unknown, invalid: false };
  } catch {
    return { invalid: true };
  }
}

export function RunWebhookDialog({ webhook }: RunWebhookDialogProps) {
  const t = useTranslations("webhooks.run");
  const common = useTranslations("common");
  const [open, setOpen] = useState(false);
  const [values, setValues] = useState<Record<string, string>>({});
  const [bodyText, setBodyText] = useState("");
  const body = bodyOf(bodyText);
  const [problems, setProblems] = useState<Record<string, string>>({});
  const [refusal, setRefusal] = useState<string | null>(null);
  const [outcome, setOutcome] = useState<string | null>(null);
  const run = useRunWebhook();
  const missing = webhook.variables.filter((name) => (values[name] ?? "").trim() === "");
  const start = async () => {
    setProblems({});
    setRefusal(null);
    setOutcome(null);
    try {
      const answer = await run.mutateAsync({ id: webhook.id, variables: values, body: body.value });
      setOutcome(answer.run_id ? t("queued", { id: answer.run_id }) : t("published"));
    } catch (error) {
      if (error instanceof ValidationError) {
        setProblems(Object.fromEntries(error.fields.map((field) => [field.field, field.message])));
        return;
      }
      setRefusal(error instanceof RequestError ? error.message : String(error));
    }
  };
  return (
    <Dialog open={open} onOpenChange={setOpen}>
      <Button type="button" variant="ghost" size="icon" aria-label={t("button")} title={t("button")} onClick={() => setOpen(true)}>
        <Play aria-hidden />
      </Button>
      <DialogContent closeLabel={common("close")}>
        <DialogHeader>
          <DialogTitle>{t("title", { title: webhook.title })}</DialogTitle>
          <DialogDescription>{t("description")}</DialogDescription>
        </DialogHeader>
        <div className="grid gap-4">
          {webhook.variables.map((name) => (
            <FormField key={name} id={`run-${webhook.id}-${name}`} label={name} error={problems[name]}>
              <Input
                id={`run-${webhook.id}-${name}`}
                autoComplete="off"
                spellCheck={false}
                value={values[name] ?? ""}
                onChange={(event) => setValues((current) => ({ ...current, [name]: event.target.value }))}
              />
            </FormField>
          ))}
          <FormField id={`run-${webhook.id}-body`} label={t("body")} hint={t("bodyHint")} optional error={body.invalid ? t("bodyInvalid") : undefined}>
            <Textarea
              id={`run-${webhook.id}-body`}
              spellCheck={false}
              value={bodyText}
              onChange={(event) => setBodyText(event.target.value)}
            />
          </FormField>
          {outcome ? (
            <p role="status" className="text-sm">
              {outcome}
            </p>
          ) : null}
          {refusal ? (
            <p role="alert" className="text-sm text-destructive">
              {refusal}
            </p>
          ) : null}
        </div>
        <DialogFooter>
          <Button type="button" disabled={run.isPending || missing.length > 0 || body.invalid} onClick={() => void start()}>
            {t("submit")}
          </Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  );
}
