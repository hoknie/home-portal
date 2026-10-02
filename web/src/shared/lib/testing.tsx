import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { act, render } from "@testing-library/react";
import type { ReactElement } from "react";
import { vi } from "vitest";

import { type Locale, TestIntl } from "@/shared/i18n";

export const TEST_SESSION_KEY = ["session"] as const;

export const TEST_ADMIN = { name: "admin", group: "admin", admin: true, rights: {} } as const;

export type TestClientOptions = { signedIn?: boolean };

export function testQueryClient({ signedIn = true }: TestClientOptions = {}) {
  const client = new QueryClient({ defaultOptions: { queries: { retry: false, refetchInterval: false }, mutations: { retry: false } } });
  if (signedIn) {
    client.setQueryData(TEST_SESSION_KEY, TEST_ADMIN);
  }
  return client;
}

export type RenderOptions = { locale?: Locale };

export function renderWithProviders(element: ReactElement, client: QueryClient = testQueryClient(), options: RenderOptions = {}) {
  const wrapped = (shown: ReactElement) => (
    <QueryClientProvider client={client}>
      <TestIntl locale={options.locale}>{shown}</TestIntl>
    </QueryClientProvider>
  );
  const result = render(wrapped(element));
  return { ...result, client, rerender: (shown: ReactElement) => result.rerender(wrapped(shown)) };
}

export function jsonResponse(body: unknown, init: { status?: number; headers?: Record<string, string> } = {}) {
  return new Response(JSON.stringify(body), {
    status: init.status ?? 200,
    headers: { "Content-Type": "application/json", ...init.headers },
  });
}

export const TRACE_ENTRY_BLANKS = {
  label: "",
  iteration: null,
  detail: "",
  output: null,
  shape: null,
  stdout: null,
  stderr: null,
  command: null,
  budget_reached: false,
  values: [],
  log: [],
  values_dropped: 0,
  log_dropped: 0,
  item: null,
  level: null,
  wait_seconds: null,
} as const;

export async function renderWhileLoading(element: ReactElement) {
  const fetch = vi.fn(() => new Promise<Response>(() => undefined));
  vi.stubGlobal("fetch", fetch);
  const result = renderWithProviders(element);
  await act(async () => {
    await new Promise((resolve) => setTimeout(resolve, 20));
  });
  return { ...result, fetched: fetch.mock.calls.length > 0 };
}

export function loadingProblems(container: HTMLElement, fetched: boolean): string[] {
  const problems: string[] = [];
  if (container.childElementCount === 0) {
    problems.push("renders nothing while its data loads");
  }
  if (fetched && container.querySelector("[data-skeleton]") === null) {
    problems.push("waits for data without a skeleton");
  }
  return problems;
}
