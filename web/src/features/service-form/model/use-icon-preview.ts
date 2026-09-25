"use client";

import { useEffect, useState } from "react";

import { FETCHED, LUCIDE_PREFIX, previewIcon } from "@/entities/service";
import { ValidationError } from "@/shared/api";

export const PREVIEW_DELAY_MILLISECONDS = 500;

export const AUTO = "auto";

export type IconPreview = { name: string | null; source: string | null; loading: boolean; problem: string | null; flush: () => void };

type Wanted = { icon: string; url: string | null };

type Result = { key: string; source: string | null; problem: string | null };

export function wantedPreview(icon: string, url: string): Wanted | null {
  const value = icon.trim();
  if (value === "" || value.startsWith(LUCIDE_PREFIX) || !FETCHED.some((prefix) => value === prefix || value.startsWith(prefix))) {
    return null;
  }
  return { icon: value, url: value === AUTO ? url.trim() : null };
}

export function lucideName(icon: string) {
  const value = icon.trim();
  if (value === "") {
    return null;
  }
  return value.startsWith(LUCIDE_PREFIX) ? value.slice(LUCIDE_PREFIX.length) : value;
}

function problemOf(error: unknown) {
  if (error instanceof ValidationError) {
    return (error.fields.find((field) => field.field === "icon") ?? error.fields[0])?.message ?? error.message;
  }
  return error instanceof Error ? error.message : String(error);
}

export function useIconPreview(icon: string, url: string): IconPreview {
  const wanted = wantedPreview(icon, url);
  const key = wanted ? JSON.stringify(wanted) : null;
  const [settled, setSettled] = useState<string | null>(null);
  const [result, setResult] = useState<Result | null>(null);
  const active = key !== null && settled === key ? key : null;

  useEffect(() => {
    if (key === null) {
      return;
    }
    const timer = setTimeout(() => setSettled(key), PREVIEW_DELAY_MILLISECONDS);
    return () => clearTimeout(timer);
  }, [key]);

  useEffect(() => {
    if (active === null) {
      return;
    }
    const controller = new AbortController();
    let objectUrl: string | null = null;
    const { icon: source, url: address } = JSON.parse(active) as Wanted;
    previewIcon(source, address, controller.signal).then(
      (blob) => {
        if (controller.signal.aborted) {
          return;
        }
        objectUrl = URL.createObjectURL(blob);
        setResult({ key: active, source: objectUrl, problem: null });
      },
      (error: unknown) => {
        if (!controller.signal.aborted) {
          setResult({ key: active, source: null, problem: problemOf(error) });
        }
      },
    );
    return () => {
      controller.abort();
      if (objectUrl) {
        URL.revokeObjectURL(objectUrl);
      }
    };
  }, [active]);

  const flush = () => setSettled(key);
  if (key === null) {
    return { name: lucideName(icon), source: null, loading: false, problem: null, flush };
  }
  const current = result?.key === key ? result : null;
  return { name: null, source: current?.source ?? null, loading: current === null, problem: current?.problem ?? null, flush };
}
