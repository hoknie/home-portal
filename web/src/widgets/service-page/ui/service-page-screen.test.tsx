import { fireEvent, screen, within } from "@testing-library/react";
import { afterEach, expect, it, vi } from "vitest";

import { apiSamples } from "@/shared/api";
import { jsonResponse, renderWithProviders } from "@/shared/lib/testing";

import { ServicePageScreen } from "./service-page-screen";

let id = "media";

vi.mock("next/navigation", () => ({
  useSearchParams: () => new URLSearchParams(`id=${id}`),
  usePathname: () => "/service/",
  useRouter: () => ({ replace: vi.fn(), push: vi.fn() }),
}));

type Services = { services: Array<{ notes: string | null; status: Record<string, unknown> }> };

const copy = (): Services => structuredClone(apiSamples.services) as Services;

function serve(services: Services = copy()) {
  const fetch = vi.fn(async (input: RequestInfo | URL) => {
    const path = String(input);
    if (path.startsWith("/api/services/") && path.includes("/history")) {
      const range = new URL(path, "http://portal").searchParams.get("range") ?? "24h";
      const days = { "24h": 1, "7d": 7, "30d": 30 }[range] ?? 1;
      const to = Date.parse(apiSamples.history.to);
      return jsonResponse({ ...apiSamples.history, range, from: new Date(to - days * 86_400_000).toISOString() });
    }
    if (path === "/api/services") {
      return jsonResponse(services, { headers: { ETag: '"r"' } });
    }
    if (path === "/api/environment") {
      return jsonResponse(apiSamples.environment);
    }
    if (path.startsWith("/api/dashboard")) {
      return jsonResponse({ sections: [{ id: "main", title: null }], widgets: [{ ...apiSamples.dashboard.widgets[1], key: "box", id: "box", type: "host-metrics", settings: {} }] });
    }
    if (path.startsWith("/api/widgets/")) {
      return jsonResponse(apiSamples.widgetHostMetrics);
    }
    return new Response("missing", { status: 404 });
  });
  vi.stubGlobal("fetch", fetch);
  renderWithProviders(<ServicePageScreen />);
  return fetch;
}

afterEach(() => {
  vi.unstubAllGlobals();
  id = "media";
});

it("shows the service, a control that opens it at the visitor's address and one that edits it", async () => {
  serve();
  expect(await screen.findByRole("heading", { level: 1, name: "Media" })).toBeInTheDocument();
  expect(screen.getByRole("link", { name: "Открыть сервис" })).toHaveAttribute("href", "http://192.168.1.10:8096");
  expect(screen.getByRole("link", { name: "Изменить" }).getAttribute("href")).toMatch(/^\/admin\/services\/edit\/?\?id=media$/);
  expect(screen.getByRole("button", { name: "Проверить сейчас" })).toBeEnabled();
});

it("lists every address, marking the visitor's and the probed one", async () => {
  serve();
  const internet = await screen.findByText("https://media.example.com");
  const local = screen.getAllByText("http://192.168.1.10:8096").find((element) => element.closest("[data-environment]"));
  const row = local?.closest("li");
  expect(row).toBeTruthy();
  expect(within(row as HTMLElement).getByText("проверяется")).toBeInTheDocument();
  expect(internet.closest("li")?.textContent).not.toContain("проверяется");
});

it("shows the probe settings and history: uptime for three ranges, the latency chart and the state changes", async () => {
  serve();
  expect(await screen.findByText("HTTP-запрос")).toBeInTheDocument();
  expect(screen.getByText("каждые 30 с")).toBeInTheDocument();
  expect(await screen.findByText("Доступность за 24 часа")).toBeInTheDocument();
  expect(screen.getByText("Доступность за 30 дней")).toBeInTheDocument();
  expect(screen.getAllByText(/^80\s%$/)).toHaveLength(3);
  expect(screen.getByRole("img", { name: "Задержка за 24 часа" })).toBeInTheDocument();
  expect(screen.getByText("connection refused", { selector: "p" })).toBeInTheDocument();
});

it("draws time and millisecond axes: hours for a day, dates for a month", async () => {
  serve();
  const chart = await screen.findByRole("img", { name: "Задержка за 24 часа" });
  const card = chart.closest("section") ?? document.body;
  const hours = [...card.querySelectorAll("[data-tick]")].map((tick) => tick.textContent ?? "");
  expect(hours.length).toBeGreaterThanOrEqual(2);
  expect(hours.every((label) => /^\d{1,2}:\d{2}$/.test(label))).toBe(true);
  expect(card.querySelector('[data-value-tick="0"]')?.textContent).toBe("0 мс");
  fireEvent.click(screen.getByRole("button", { name: "30 дней" }));
  await screen.findByRole("img", { name: "Задержка за 30 дней" });
  const days = [...card.querySelectorAll("[data-tick]")].map((tick) => tick.textContent ?? "");
  expect(days.length).toBeGreaterThanOrEqual(2);
  expect(days.every((label) => /^\d{1,2} \S+$/.test(label))).toBe(true);
});

it("renders links and notes, and a script in the notes is shown as text", async () => {
  const services = copy();
  services.services[0].notes = "Restart it like this.\n\n<script>alert(1)</script>";
  serve(services);
  expect(await screen.findByRole("link", { name: "Admin" })).toHaveAttribute("href", "http://192.168.1.10:8096/web/#/dashboard");
  expect(screen.getByText("<script>alert(1)</script>")).toBeInTheDocument();
  expect(document.querySelector("script")).toBeNull();
});

it("draws the widgets tied to the service", async () => {
  serve();
  expect(await screen.findByRole("heading", { level: 2, name: "Виджеты сервиса" })).toBeInTheDocument();
});

it("explains a failure and what to do about it", async () => {
  id = "nas";
  serve();
  const alert = await screen.findByRole("alert");
  expect(within(alert).getByText("Соединение отклонено")).toBeInTheDocument();
});

it("names the macOS setting when the operating system refused the local network", async () => {
  id = "nas";
  const services = copy();
  services.services[1].status = { ...services.services[1].status, state: "unreadable", diagnosis: "local-network-denied" };
  serve(services);
  const alert = await screen.findByRole("alert");
  expect(within(alert).getByText(/Конфиденциальность и безопасность → Локальная сеть/)).toBeInTheDocument();
});

it("says a service that is unknown here was not found", async () => {
  id = "secret";
  serve();
  expect(await screen.findByText("Сервис не найден")).toBeInTheDocument();
  expect(screen.getByRole("link", { name: "На главную" }).getAttribute("href")).toBe("/");
});
