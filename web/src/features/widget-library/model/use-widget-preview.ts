"use client";

import { keepPreviousData, useMutation, useQuery } from "@tanstack/react-query";
import { useEffect, useState } from "react";

import { type WidgetPreview, previewWidget } from "@/entities/widget";
import { RequestError } from "@/shared/api";

export const PREVIEW_PAUSE = 400;
export const LARGEST_SAMPLE = 16 * 1024;
export const FORBIDDEN = 403;

export type WidgetPreviewState = {
  preview: WidgetPreview | null;
  problem: string | null;
  running: boolean;
  forbidden: boolean;
  run: () => void;
};

export function useWidgetPreview(id: string | null, settings: Record<string, unknown>, enabled: boolean): WidgetPreviewState {
  const text = JSON.stringify(settings);
  const [settled, setSettled] = useState(text);
  const [sample, setSample] = useState<string | null>(null);
  useEffect(() => {
    const timer = setTimeout(() => setSettled(text), PREVIEW_PAUSE);
    return () => clearTimeout(timer);
  }, [text]);
  const query = useQuery({
    queryKey: ["widget-preview", id, settled, sample],
    queryFn: () =>
      previewWidget({
        id,
        settings: JSON.parse(settled) as Record<string, unknown>,
        run: false,
        sample: sample === null ? null : (JSON.parse(sample) as unknown),
      }),
    enabled,
    placeholderData: keepPreviousData,
    retry: false,
    staleTime: Infinity,
  });
  const runner = useMutation({
    mutationFn: () => previewWidget({ id, settings: JSON.parse(text) as Record<string, unknown>, run: true, sample: null }),
    onSuccess: (answer) => {
      const data = JSON.stringify(answer.data ?? null);
      if (answer.ran && answer.problem === null && data.length <= LARGEST_SAMPLE) {
        setSample(data);
      }
    },
  });
  const ran = runner.data ?? null;
  const tooLarge = ran !== null && ran.problem === null && JSON.stringify(ran.data ?? null).length > LARGEST_SAMPLE;
  const forbidden = runner.error instanceof RequestError && runner.error.status === FORBIDDEN;
  return {
    preview: tooLarge ? ran : (query.data ?? ran),
    problem: ran?.problem ?? null,
    running: runner.isPending,
    forbidden,
    run: () => runner.mutate(),
  };
}
