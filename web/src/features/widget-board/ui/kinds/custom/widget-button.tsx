"use client";

import { useQueryClient } from "@tanstack/react-query";
import { useRouter } from "next/navigation";
import { useTranslations } from "next-intl";
import { useEffect, useState } from "react";
import { toast } from "sonner";

import { isActive, useRun } from "@/entities/automation";
import { useCan } from "@/entities/session";
import { type RenderedLeaf, pressWidgetAction, widgetKey } from "@/entities/widget";
import { RequestError, ThrottledError } from "@/shared/api";
import { cn } from "@/shared/lib/cn";
import { routes } from "@/shared/config";
import { ConfirmDialog } from "@/shared/ui/confirm-dialog";
import { Button, toneOf } from "@/shared/ui/kit";

import { BlockIcon } from "./block-icon";

export type ButtonLeaf = Extract<RenderedLeaf, { kind: "button" }>;

const VARIANTS = { primary: "default", secondary: "outline", ghost: "ghost" } as const;

const TONED = { primary: "solid", secondary: "outline", ghost: "ghost" } as const;

const FORBIDDEN = 403;

function RunWatcher({ run, widget, onDone }: { run: string; widget: string; onDone: () => void }) {
  const query = useRun(run);
  const client = useQueryClient();
  const finished = query.data !== undefined && !isActive(query.data);
  useEffect(() => {
    if (finished) {
      void client.invalidateQueries({ queryKey: widgetKey(widget, "private") });
      onDone();
    }
  }, [finished, client, widget, onDone]);
  return null;
}

export function WidgetButton({ button, widget }: { button: ButtonLeaf; widget: string }) {
  const t = useTranslations("widgets.custom");
  const can = useCan();
  const router = useRouter();
  const client = useQueryClient();
  const [asking, setAsking] = useState(false);
  const [pending, setPending] = useState(false);
  const [watching, setWatching] = useState<string | null>(null);
  const variant = VARIANTS[button.style];
  const toned = button.tone ? toneOf(button.tone)[TONED[button.style]] : "";
  const content = (
    <>
      <BlockIcon name={button.icon} />
      {button.label}
    </>
  );
  if (button.does === "link") {
    return button.link ? (
      <Button asChild variant={variant} size="sm" className={cn("w-fit", toned)}>
        <a href={button.link} target="_blank" rel="noreferrer noopener">
          {content}
        </a>
      </Button>
    ) : (
      <Button variant={variant} size="sm" className={cn("w-fit", toned)} disabled title={t("badLink")}>
        {content}
      </Button>
    );
  }
  const needed = button.does === "automation" ? "automations" : button.does === "workflow" ? "workflows" : null;
  const allowed = needed === null || can(needed, "execute");
  const press = async () => {
    setPending(true);
    try {
      const acted = await pressWidgetAction(widget, button.action);
      if (acted.run_id) {
        const run = acted.run_id;
        setWatching(run);
        toast.success(t("queued", { label: button.label }), { action: { label: t("openRun"), onClick: () => router.push(routes.run(run)) } });
      } else {
        await client.invalidateQueries({ queryKey: widgetKey(widget, "private") });
        toast.success(t("refreshing"));
      }
    } catch (error) {
      if (error instanceof ThrottledError) {
        toast.error(t("throttled", { seconds: error.retryAfterSeconds }));
      } else if (error instanceof RequestError && error.status === FORBIDDEN) {
        toast.error(t("forbidden"));
      } else {
        toast.error(t("failed"));
      }
    } finally {
      setPending(false);
      setAsking(false);
    }
  };
  return (
    <>
      <Button
        type="button"
        variant={variant}
        size="sm"
        className={cn("w-fit", toned)}
        disabled={!allowed || pending}
        title={allowed ? undefined : t("needsRight", { area: needed ?? "" })}
        onClick={() => (button.confirm ? setAsking(true) : void press())}
        data-action={button.action}
      >
        {content}
      </Button>
      {button.confirm ? (
        <ConfirmDialog
          open={asking}
          onOpenChange={setAsking}
          title={button.confirm}
          description={t("confirmHint")}
          confirmLabel={button.label}
          pending={pending}
          onConfirm={press}
        />
      ) : null}
      {watching ? <RunWatcher run={watching} widget={widget} onDone={() => setWatching(null)} /> : null}
    </>
  );
}
