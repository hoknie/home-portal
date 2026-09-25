import { screen, waitFor } from "@testing-library/react";
import { afterEach, expect, it, vi } from "vitest";

import { apiSamples } from "@/shared/api";
import { jsonResponse, renderWithProviders } from "@/shared/lib/testing";

import { BoardWidget } from "./board-widget";

const weather = { type: "weather", id: "riga", title: null, settings: {} };

function renderWith(answer: () => Response, scope: "private" | "public" = "private") {
  const fetch = vi.fn(async (input: RequestInfo | URL) => {
    void input;
    return answer();
  });
  vi.stubGlobal("fetch", fetch);
  renderWithProviders(<BoardWidget widget={weather} services={[]} scope={scope} />);
  return fetch;
}

afterEach(() => {
  vi.unstubAllGlobals();
});

it("asks the portal for the widget data and renders it under the widget title", async () => {
  const fetch = renderWith(() => jsonResponse(apiSamples.widgetWeather));
  expect(await screen.findByText("12°C")).toBeInTheDocument();
  expect(screen.getByRole("heading", { level: 2, name: "Погода" })).toBeInTheDocument();
  expect(fetch.mock.calls[0][0]).toBe("/api/widgets/riga/data");
});

it("asks the public half for a widget of the public portal", async () => {
  const fetch = renderWith(() => jsonResponse(apiSamples.widgetWeather), "public");
  await screen.findByText("12°C");
  expect(fetch.mock.calls[0][0]).toBe("/api/public/widgets/riga/data");
});

it("marks data the portal could not refresh and names the problem", async () => {
  renderWith(() => jsonResponse({ ...apiSamples.widgetWeather, stale: true, problem: "open-meteo answered 503" }));
  const stale = await screen.findByText(/устарели/);
  expect(stale).toHaveAttribute("title", "open-meteo answered 503");
  expect(screen.getByText("12°C")).toBeInTheDocument();
});

it("explains a widget whose data never arrived, keeping its title", async () => {
  renderWith(() => jsonResponse({ error: "the widget has no data yet" }, { status: 502 }));
  await waitFor(() => expect(screen.getByRole("alert")).toBeInTheDocument());
  expect(screen.getByRole("heading", { level: 2, name: "Погода" })).toBeInTheDocument();
});

it("shows a placeholder for a type it does not know and for settings it cannot read", () => {
  renderWithProviders(<BoardWidget widget={{ type: "traffic", id: null, title: null, settings: {} }} services={[]} />);
  expect(screen.getByText("Виджет «traffic» не поддерживается этой версией интерфейса")).toBeInTheDocument();
  renderWithProviders(
    <BoardWidget widget={{ type: "services", id: null, title: null, settings: { groups: "Media" } }} services={[]} />,
  );
  expect(screen.getByText("Настройки виджета «services» заданы неверно")).toBeInTheDocument();
});

it("refuses a data-backed widget the configuration left without an id", () => {
  renderWithProviders(<BoardWidget widget={{ type: "weather", id: null, title: null, settings: {} }} services={[]} />);
  expect(screen.getByText("Настройки виджета «weather» заданы неверно")).toBeInTheDocument();
});
