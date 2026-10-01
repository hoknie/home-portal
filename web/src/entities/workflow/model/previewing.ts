import { type QueryClient, useQueryClient } from "@tanstack/react-query";
import { useEffect, useRef, useState } from "react";

import { ValidationError } from "@/shared/api";

import { previewTransform } from "../api/transform-preview";
import type { PreviewAnswer, PreviewCache, PreviewQuestion, PreviewState } from "./transforming/previews";
import { canonical } from "./transforming/values";

export const TRANSFORM_PREVIEW = "transform-preview";
export const PREVIEW_DELAY_MILLISECONDS = 300;
export const MOST_PREVIEWS_AT_ONCE = 24;

const SAME_RENDER_MILLISECONDS = 50;

type Wanted = { question: PreviewQuestion; at: number };

export function previewKey(question: PreviewQuestion) {
  return [TRANSFORM_PREVIEW, canonical(question)] as const;
}

function stateOf(client: QueryClient, question: PreviewQuestion): PreviewState | null {
  const query = client.getQueryCache().find({ queryKey: previewKey(question), exact: true });
  if (query?.state.status === "success") {
    return { state: "ready", answer: query.state.data as PreviewAnswer };
  }
  if (query?.state.status === "error") {
    const error = query.state.error;
    return error instanceof ValidationError ? { state: "refused", message: error.message } : { state: "unreachable" };
  }
  return query === undefined ? null : { state: "computing" };
}

export function usePreviewCache(): PreviewCache {
  const client = useQueryClient();
  const [, setVersion] = useState(0);
  const wanted = useRef(new Map<string, Wanted>());
  const timer = useRef<ReturnType<typeof setTimeout> | null>(null);
  useEffect(
    () =>
      client.getQueryCache().subscribe((event) => {
        if (event.query.queryKey[0] === TRANSFORM_PREVIEW && event.type === "updated") {
          setVersion((current) => current + 1);
        }
      }),
    [client],
  );
  useEffect(
    () => () => {
      if (timer.current !== null) {
        clearTimeout(timer.current);
      }
    },
    [],
  );
  const ask = () => {
    timer.current = null;
    const entries = [...wanted.current.values()];
    const latest = Math.max(...entries.map((entry) => entry.at));
    const due = entries.filter((entry) => entry.at >= latest - SAME_RENDER_MILLISECONDS).slice(0, MOST_PREVIEWS_AT_ONCE);
    wanted.current.clear();
    for (const { question } of due) {
      void client.prefetchQuery({ queryKey: previewKey(question), queryFn: () => previewTransform(question), staleTime: Infinity, retry: false });
    }
  };
  return (question) => {
    const known = stateOf(client, question);
    if (known !== null) {
      return known;
    }
    wanted.current.set(canonical(question), { question, at: Date.now() });
    if (timer.current !== null) {
      clearTimeout(timer.current);
    }
    timer.current = setTimeout(ask, PREVIEW_DELAY_MILLISECONDS);
    return { state: "computing" };
  };
}
