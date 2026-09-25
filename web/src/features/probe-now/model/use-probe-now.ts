"use client";

import { useQueryClient } from "@tanstack/react-query";
import { useEffect, useState } from "react";

import { type Service, servicesKey, useProbeService } from "@/entities/service";
import { ThrottledError } from "@/shared/api";

export const POLL_MILLISECONDS = 1_000;
export const GRACE_MILLISECONDS = 2_000;

type Waiting = { since: string | null; deadline: number };

export function useProbeNow(service: Service) {
  const client = useQueryClient();
  const probe = useProbeService();
  const [waiting, setWaiting] = useState<Waiting | null>(null);
  const [retryAfter, setRetryAfter] = useState<number | null>(null);
  const checkedAt = service.status.checked_at;

  const pending = waiting !== null && checkedAt === waiting.since;

  useEffect(() => {
    if (!waiting || !pending) {
      return;
    }
    const timer = window.setInterval(() => {
      if (Date.now() > waiting.deadline) {
        setWaiting(null);
        return;
      }
      void client.invalidateQueries({ queryKey: servicesKey });
    }, POLL_MILLISECONDS);
    return () => window.clearInterval(timer);
  }, [waiting, pending, client]);

  const start = () => {
    setRetryAfter(null);
    probe.mutate(service.id, {
      onSuccess: () =>
        setWaiting({
          since: checkedAt,
          deadline: Date.now() + service.probe.timeout_seconds * 1000 + GRACE_MILLISECONDS,
        }),
      onError: (error) => {
        if (error instanceof ThrottledError) {
          setRetryAfter(error.retryAfterSeconds);
        }
      },
    });
  };

  return {
    start,
    busy: probe.isPending || pending,
    retryAfter,
    failed: probe.isError && !(probe.error instanceof ThrottledError),
    disabled: !service.probe.enabled,
  };
}
