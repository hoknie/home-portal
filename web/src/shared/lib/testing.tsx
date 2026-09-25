import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { render } from "@testing-library/react";
import type { ReactElement } from "react";

import { type Locale, TestIntl } from "@/shared/i18n";

export function testQueryClient() {
  return new QueryClient({ defaultOptions: { queries: { retry: false, refetchInterval: false }, mutations: { retry: false } } });
}

export type RenderOptions = { locale?: Locale };

export function renderWithProviders(element: ReactElement, client: QueryClient = testQueryClient(), options: RenderOptions = {}) {
  const result = render(
    <QueryClientProvider client={client}>
      <TestIntl locale={options.locale}>{element}</TestIntl>
    </QueryClientProvider>,
  );
  return { ...result, client };
}

export function jsonResponse(body: unknown, init: { status?: number; headers?: Record<string, string> } = {}) {
  return new Response(JSON.stringify(body), {
    status: init.status ?? 200,
    headers: { "Content-Type": "application/json", ...init.headers },
  });
}
