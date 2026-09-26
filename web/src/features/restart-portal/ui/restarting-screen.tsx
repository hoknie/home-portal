"use client";

import { LoaderCircle, TriangleAlert } from "lucide-react";
import { useTranslations } from "next-intl";
import { useEffect, useState } from "react";

import { api } from "@/shared/config";
import { Button } from "@/shared/ui/primitives";

export const POLL_MILLISECONDS = 1_000;
export const GIVE_UP_MILLISECONDS = 60_000;
export const DOWN_AT_MOST_MILLISECONDS = 5_000;

export type RestartingScreenProps = { target: string };

async function answers(target: string, sameOrigin: boolean) {
  try {
    const response = await fetch(`${target}${api.health}`, { cache: "no-store", mode: sameOrigin ? "same-origin" : "no-cors" });
    return sameOrigin ? response.ok : true;
  } catch {
    return false;
  }
}

export function RestartingScreen({ target }: RestartingScreenProps) {
  const t = useTranslations("restart");
  const [attempt, setAttempt] = useState(0);
  const [gaveUp, setGaveUp] = useState(false);
  useEffect(() => {
    const sameOrigin = target === window.location.origin;
    const began = Date.now();
    let wentDown = !sameOrigin;
    let stopped = false;
    const poll = async () => {
      if (stopped) {
        return;
      }
      const elapsed = Date.now() - began;
      if (elapsed >= GIVE_UP_MILLISECONDS) {
        setGaveUp(true);
        return;
      }
      const up = await answers(target, sameOrigin);
      if (stopped) {
        return;
      }
      if (!up) {
        wentDown = true;
      } else if (wentDown || elapsed >= DOWN_AT_MOST_MILLISECONDS) {
        if (sameOrigin) {
          window.location.reload();
        } else {
          const destination = new URL(`${window.location.pathname}${window.location.search}`, target).href;
          window.location.assign(destination);
        }
        return;
      }
      timer = window.setTimeout(() => void poll(), POLL_MILLISECONDS);
    };
    let timer = window.setTimeout(() => void poll(), POLL_MILLISECONDS);
    return () => {
      stopped = true;
      window.clearTimeout(timer);
    };
  }, [target, attempt]);
  const retry = () => {
    setGaveUp(false);
    setAttempt((current) => current + 1);
  };
  return (
    <div role="alertdialog" aria-live="polite" aria-label={t("restarting")} className="fixed inset-0 z-50 grid place-items-center bg-background/90 p-6 backdrop-blur-sm">
      <div className="grid max-w-sm justify-items-center gap-3 text-center">
        {gaveUp ? <TriangleAlert className="size-8 text-status-degraded" aria-hidden /> : <LoaderCircle className="size-8 animate-spin" aria-hidden />}
        <p className="text-lg font-medium">{gaveUp ? t("timeout") : t("restarting")}</p>
        <p className="text-sm text-muted-foreground">{gaveUp ? t("timeoutHint") : t("restartingHint")}</p>
        {gaveUp ? (
          <Button type="button" onClick={retry}>
            {t("retry")}
          </Button>
        ) : null}
      </div>
    </div>
  );
}
