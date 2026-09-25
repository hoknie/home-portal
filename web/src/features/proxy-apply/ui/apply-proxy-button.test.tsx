import { screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { afterEach, expect, it, vi } from "vitest";

import { renderWithProviders } from "@/shared/lib/testing";
import { Toaster } from "@/shared/ui/primitives";

import { ApplyProxyButton } from "./apply-proxy-button";

afterEach(() => {
  vi.unstubAllGlobals();
});

it("reports caddy's own refusal", async () => {
  vi.stubGlobal("fetch", vi.fn(async () => new Response("Caddy refused with 400: unknown module", { status: 502 })));
  renderWithProviders(
    <>
      <ApplyProxyButton />
      <Toaster />
    </>,
  );
  await userEvent.click(screen.getByRole("button", { name: "Применить сейчас" }));
  expect(await screen.findByText(/Caddy не принял конфигурацию: Caddy refused with 400: unknown module/)).toBeInTheDocument();
});
