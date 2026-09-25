import type { z } from "zod";

import { routes } from "@/shared/config";

import { ConflictError, RequestError, ThrottledError, UnauthorizedError, UnreachableError, ValidationError } from "./errors";
import { fieldErrorsSchema } from "./schemas";

export type Revisioned<T> = { data: T; revision: string | null };

export type RequestOptions<T> = {
  method?: "GET" | "POST" | "PUT" | "DELETE";
  body?: unknown;
  revision?: string | null;
  schema: z.ZodType<T>;
  redirectOnUnauthorized?: boolean;
};

export const NEXT_PARAMETER = "next";

export function signInLocation(current: { pathname: string; search: string }) {
  const next = `${current.pathname}${current.search}`;
  return `${routes.login}?${NEXT_PARAMETER}=${encodeURIComponent(next)}`;
}

export async function request<T>(path: string, options: RequestOptions<T>): Promise<Revisioned<T>> {
  const headers: Record<string, string> = { Accept: "application/json" };
  if (options.body !== undefined) {
    headers["Content-Type"] = "application/json";
  }
  if (options.revision) {
    headers["If-Match"] = options.revision;
  }
  let response: Response;
  try {
    response = await fetch(path, {
      method: options.method ?? "GET",
      headers,
      credentials: "same-origin",
      body: options.body === undefined ? undefined : JSON.stringify(options.body),
    });
  } catch (cause) {
    throw new UnreachableError(cause);
  }
  if (!response.ok) {
    throw await failure(response, options.redirectOnUnauthorized ?? true);
  }
  const revision = response.headers.get("ETag");
  const text = await response.text();
  const payload: unknown = text ? JSON.parse(text) : null;
  return { data: options.schema.parse(payload), revision };
}

export async function requestImage(path: string, body: unknown, signal?: AbortSignal): Promise<Blob> {
  let response: Response;
  try {
    response = await fetch(path, {
      method: "POST",
      headers: { Accept: "image/*", "Content-Type": "application/json" },
      credentials: "same-origin",
      body: JSON.stringify(body),
      signal,
    });
  } catch (cause) {
    if (signal?.aborted) {
      throw cause;
    }
    throw new UnreachableError(cause);
  }
  if (!response.ok) {
    throw await failure(response, true);
  }
  return response.blob();
}

async function failure(response: Response, redirectOnUnauthorized: boolean): Promise<Error> {
  const text = await response.text();
  switch (response.status) {
    case 401:
      if (redirectOnUnauthorized && typeof window !== "undefined" && window.location.pathname !== routes.login) {
        window.location.assign(signInLocation(window.location));
      }
      return new UnauthorizedError(text);
    case 409:
      return new ConflictError(text);
    case 422: {
      const parsed = fieldErrorsSchema.safeParse(safeJson(text));
      return parsed.success ? new ValidationError(parsed.data.errors) : new RequestError(422, text);
    }
    case 429:
      return new ThrottledError(Number(response.headers.get("Retry-After") ?? "60"), text);
    default:
      return new RequestError(response.status, text);
  }
}

function safeJson(text: string): unknown {
  try {
    return JSON.parse(text);
  } catch {
    return null;
  }
}
