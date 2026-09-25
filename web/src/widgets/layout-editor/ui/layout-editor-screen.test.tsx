import { act, fireEvent, screen, waitFor, within } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { afterEach, beforeEach, expect, it, vi } from "vitest";

import { apiSamples } from "@/shared/api";
import { jsonResponse, renderWithProviders } from "@/shared/lib/testing";

import { LayoutEditorScreen } from "./layout-editor-screen";

vi.mock("next/navigation", () => ({ usePathname: () => "/admin/layout/", useRouter: () => ({ replace: vi.fn(), push: vi.fn() }) }));

const LAYOUT = {
  sections: [
    { id: "now", title: "Сейчас" },
    { id: "media", title: "Медиа" },
  ],
  widgets: [
    { key: "status-summary", type: "status-summary", id: "status-summary", title: null, settings: {}, section: "now", size: "two-thirds", environments: null, public: false },
    { key: "riga", type: "weather", id: "riga", title: null, settings: { latitude: 56.95, longitude: 24.11 }, section: "now", size: "third", environments: null, public: true },
    { key: "roads", type: "traffic", id: "roads", title: null, settings: { city: "Riga", zoom: 12 }, section: "media", size: "full", environments: ["local"], public: false },
  ],
};

type Put = { body: unknown; revision: string | null };

function serve(answer: (put: Put) => Response = () => jsonResponse(LAYOUT, { headers: { ETag: '"r2"' } })) {
  const puts: Put[] = [];
  const fetch = vi.fn(async (input: RequestInfo | URL, init?: RequestInit) => {
    const path = String(input);
    if (init?.method === "PUT") {
      const put = { body: JSON.parse(String(init.body)), revision: new Headers(init.headers).get("If-Match") };
      puts.push(put);
      return answer(put);
    }
    if (path === "/api/dashboard?all=true") {
      return jsonResponse(LAYOUT, { headers: { ETag: '"r1"' } });
    }
    if (path === "/api/environment") {
      return jsonResponse({ environment: "local", detected: "local", switchable: true, environments: ["local", "vpn", "internet"] });
    }
    if (path === "/api/services") {
      const services = structuredClone(apiSamples.services) as { services: Array<{ group: string | null }> };
      services.services[1].group = "Network";
      return jsonResponse(services, { headers: { ETag: '"r1"' } });
    }
    return jsonResponse(apiSamples.widgetWeather);
  });
  vi.stubGlobal("fetch", fetch);
  renderWithProviders(<LayoutEditorScreen />);
  return puts;
}

const order = (section: string) =>
  [...document.querySelectorAll(`[data-section="${section}"] [data-widget]`)].map((element) => element.getAttribute("data-widget"));

beforeEach(() => {
  vi.spyOn(Element.prototype, "getBoundingClientRect").mockImplementation(function rect(this: Element) {
    if (this.matches("section[data-section-block]")) {
      const top = [...document.querySelectorAll("section[data-section-block]")].indexOf(this) * 1000;
      return { x: 0, y: top, top, left: 0, width: 1200, height: 900, right: 1200, bottom: top + 900, toJSON: () => ({}) } as DOMRect;
    }
    const tiles = [...document.querySelectorAll("[data-widget]")];
    const index = tiles.indexOf(this.closest("[data-widget]") ?? this);
    const top = index < 0 ? 0 : index * 120;
    return { x: 0, y: top, top, left: 0, width: 300, height: 100, right: 300, bottom: top + 100, toJSON: () => ({}) } as DOMRect;
  });
});

afterEach(() => {
  vi.restoreAllMocks();
  vi.unstubAllGlobals();
});

const tick = () => new Promise((resolve) => setTimeout(resolve, 10));

async function press(target: Element | Document, key: string, code = key) {
  await act(async () => {
    fireEvent.keyDown(target, { key, code });
    await tick();
  });
}

const tileOf = (key: string) => document.querySelector(`[data-widget="${key}"]`) as HTMLElement;

it("reorders a widget with the keyboard and saves the new order with the loaded revision", async () => {
  const puts = serve();
  const handle = await screen.findByRole("button", { name: "Переместить «Сводка»" });
  handle.focus();
  await act(async () => {
    fireEvent.keyDown(handle, { key: " ", code: "Space" });
    await tick();
  });
  await act(async () => {
    fireEvent.keyDown(document, { key: "ArrowDown", code: "ArrowDown" });
    await tick();
  });
  await act(async () => {
    fireEvent.keyDown(document, { key: " ", code: "Space" });
    await tick();
  });
  await waitFor(() => expect(order("now")).toEqual(["riga", "status-summary"]));
  expect(await screen.findByText("Виджет «Сводка» поставлен на позицию 2 в секции «Сейчас».")).toBeInTheDocument();
  expect(screen.getByText("Есть несохранённые изменения")).toBeInTheDocument();
  await userEvent.click(screen.getByRole("button", { name: "Сохранить раскладку" }));
  await waitFor(() => expect(puts).toHaveLength(1));
  expect(puts[0].revision).toBe('"r1"');
  const sent = puts[0].body as { widgets: Array<{ key: string; settings: unknown }> };
  expect(sent.widgets.map((widget) => widget.key)).toEqual(["riga", "status-summary", "roads"]);
  expect(sent.widgets[2].settings).toEqual({ city: "Riga", zoom: 12 });
});

it("moves a widget into another section by dragging it with the keyboard", async () => {
  const puts = serve();
  const handle = await screen.findByRole("button", { name: "Переместить «Погода»" });
  handle.focus();
  await press(handle, " ", "Space");
  await press(document, "ArrowDown");
  await press(document, " ", "Space");
  await waitFor(() => expect(order("media")).toEqual(["riga", "roads"]));
  await userEvent.click(screen.getByRole("button", { name: "Сохранить раскладку" }));
  await waitFor(() => expect(puts).toHaveLength(1));
  const sent = puts[0].body as { widgets: Array<{ key: string; section: string }> };
  expect(sent.widgets.find((widget) => widget.key === "riga")).toMatchObject({ section: "media" });
});

it("offers no list for a widget's size or section", async () => {
  serve();
  await screen.findByRole("button", { name: "Переместить «Сводка»" });
  const tile = tileOf("status-summary");
  expect(within(tile).queryByRole("combobox")).not.toBeInTheDocument();
  expect(tile.querySelector("select")).toBeNull();
  expect(screen.queryByRole("button", { name: /Поднять секцию|Опустить секцию/ })).not.toBeInTheDocument();
});

it("resizes a widget with the arrows on its edge and announces the new width", async () => {
  const puts = serve();
  const edge = await screen.findByRole("slider", { name: "Ширина виджета «Погода»" });
  expect(edge).toHaveAttribute("aria-valuetext", "треть");
  fireEvent.keyDown(edge, { key: "ArrowRight" });
  expect(tileOf("riga")).toHaveAttribute("data-size", "half");
  fireEvent.keyDown(edge, { key: "ArrowRight" });
  expect(tileOf("riga")).toHaveAttribute("data-size", "two-thirds");
  expect(document.querySelector("[data-resize-notice]")).toHaveTextContent("Ширина виджета «Погода»: две трети.");
  await userEvent.click(screen.getByRole("button", { name: "Сохранить раскладку" }));
  await waitFor(() => expect(puts).toHaveLength(1));
  const sent = puts[0].body as { widgets: Array<{ key: string; size: string }> };
  expect(sent.widgets.find((widget) => widget.key === "riga")).toMatchObject({ size: "two-thirds" });
});

it("resizes a widget by dragging its edge, snapping to the nearest width", async () => {
  serve();
  const edge = await screen.findByRole("slider", { name: "Ширина виджета «Сводка»" });
  fireEvent.pointerDown(edge, { pointerId: 1, clientX: 800 });
  fireEvent.pointerMove(edge, { pointerId: 1, clientX: 750 });
  expect(within(tileOf("status-summary")).getByText("половина")).toBeInTheDocument();
  fireEvent.pointerUp(edge, { pointerId: 1, clientX: 750 });
  expect(tileOf("status-summary")).toHaveAttribute("data-size", "half");
});

it("keeps the width when a drag of the edge is cancelled with Escape", async () => {
  serve();
  const edge = await screen.findByRole("slider", { name: "Ширина виджета «Сводка»" });
  fireEvent.pointerDown(edge, { pointerId: 1, clientX: 800 });
  fireEvent.pointerMove(edge, { pointerId: 1, clientX: 750 });
  fireEvent.keyDown(window, { key: "Escape" });
  fireEvent.pointerUp(edge, { pointerId: 1, clientX: 750 });
  expect(tileOf("status-summary")).toHaveAttribute("data-size", "two-thirds");
  expect(screen.queryByText("Есть несохранённые изменения")).not.toBeInTheDocument();
});

it("reorders sections by dragging their handle with the keyboard and saves them in the new order", async () => {
  const puts = serve();
  const handle = await screen.findByRole("button", { name: "Переместить секцию «Сейчас»" });
  handle.focus();
  await press(handle, " ", "Space");
  await press(document, "ArrowDown");
  await press(document, " ", "Space");
  await waitFor(() =>
    expect([...document.querySelectorAll("section[data-section-block]")].map((section) => section.getAttribute("data-section"))).toEqual(["media", "now"]),
  );
  expect(await screen.findByText("Секция «Сейчас» поставлена на позицию 2.")).toBeInTheDocument();
  await userEvent.click(screen.getByRole("button", { name: "Сохранить раскладку" }));
  await waitFor(() => expect(puts).toHaveLength(1));
  const sent = puts[0].body as { sections: Array<{ id: string }>; widgets: Array<{ key: string }> };
  expect(sent.sections.map((section) => section.id)).toEqual(["media", "now"]);
  expect(sent.widgets.map((widget) => widget.key)).toEqual(["roads", "status-summary", "riga"]);
});

it("asks before leaving with unsaved changes", async () => {
  serve();
  await screen.findByRole("button", { name: "Переместить «Сводка»" });
  const confirm = vi.spyOn(window, "confirm").mockReturnValue(false);
  const link = document.createElement("a");
  link.href = "/";
  link.textContent = "home";
  document.body.append(link);
  await userEvent.click(link);
  expect(confirm).not.toHaveBeenCalled();
  fireEvent.keyDown(within(tileOf("riga")).getByRole("slider"), { key: "ArrowRight" });
  await userEvent.click(link);
  expect(confirm).toHaveBeenCalledWith("Уйти со страницы? Несохранённые изменения раскладки пропадут.");
  link.remove();
});

it("previews the page as a visitor from the internet without a session sees it", async () => {
  serve();
  await screen.findByRole("button", { name: "Переместить «Сводка»" });
  await userEvent.click(screen.getByText("Предпросмотр"));
  await userEvent.selectOptions(screen.getByLabelText("Окружение посетителя"), "internet");
  await userEvent.click(screen.getByLabelText("Посетитель вошёл"));
  const preview = screen.getByText("Предпросмотр").closest("details") as HTMLElement;
  expect(within(preview).getByRole("heading", { level: 2, name: "Сейчас" })).toBeInTheDocument();
  expect(within(preview).queryByRole("heading", { level: 2, name: "Медиа" })).not.toBeInTheDocument();
  expect(within(preview).queryByText("Сводка")).not.toBeInTheDocument();
});

it("on a conflict it keeps the arrangement and offers to reload or save over", async () => {
  serve(() => new Response("stale", { status: 409 }));
  await screen.findByRole("button", { name: "Переместить «Сводка»" });
  fireEvent.keyDown(within(tileOf("riga")).getByRole("slider"), { key: "ArrowRight" });
  await userEvent.click(screen.getByRole("button", { name: "Сохранить раскладку" }));
  expect(await screen.findByText(/Файл конфигурации изменился/)).toBeInTheDocument();
  expect(screen.getByRole("button", { name: "Загрузить раскладку из файла" })).toBeInTheDocument();
  expect(screen.getByRole("button", { name: "Сохранить поверх" })).toBeInTheDocument();
  expect(document.querySelector('[data-widget="riga"]')).toHaveAttribute("data-size", "half");
});

it("shows a server error on the widget it names", async () => {
  serve(() => jsonResponse({ errors: [{ field: "widgets[1].settings.latitude", message: "must be between -90 and 90" }] }, { status: 422 }));
  await screen.findByRole("button", { name: "Переместить «Сводка»" });
  fireEvent.keyDown(within(tileOf("riga")).getByRole("slider"), { key: "ArrowRight" });
  await userEvent.click(screen.getByRole("button", { name: "Сохранить раскладку" }));
  const tile = document.querySelector('[data-widget="riga"]') as HTMLElement;
  expect(await within(tile).findByText("widgets[1].settings.latitude: must be between -90 and 90")).toBeInTheDocument();
});

it("adds a widget, edits its settings in the sheet and removes another after confirming", async () => {
  const puts = serve();
  await screen.findByRole("button", { name: "Переместить «Сводка»" });
  const media = document.querySelector('[data-section="media"]') as HTMLElement;
  await userEvent.selectOptions(within(media).getByLabelText("Тип виджета"), "calendar");
  await userEvent.click(within(media).getByRole("button", { name: "Добавить виджет" }));
  const added = order("media").at(-1) as string;
  await userEvent.click(within(document.querySelector(`[data-widget="${added}"]`) as HTMLElement).getByRole("button", { name: "Настроить" }));
  await userEvent.type(screen.getByLabelText("Адрес календаря (.ics)"), "https://calendar.example.com/home.ics");
  await userEvent.click(screen.getByRole("button", { name: "Готово" }));
  await userEvent.click(within(document.querySelector('[data-widget="riga"]') as HTMLElement).getByRole("button", { name: "Убрать" }));
  await userEvent.click(within(screen.getByRole("dialog")).getByRole("button", { name: "Убрать" }));
  await userEvent.click(screen.getByRole("button", { name: "Сохранить раскладку" }));
  await waitFor(() => expect(puts).toHaveLength(1));
  const sent = puts[0].body as { widgets: Array<{ key: string | null; type: string; settings: Record<string, unknown> }> };
  expect(sent.widgets.map((widget) => widget.type)).toEqual(["status-summary", "traffic", "calendar"]);
  expect(sent.widgets[2]).toMatchObject({ key: null, settings: { url: "https://calendar.example.com/home.ics" } });
});

it("chooses the groups of a services widget from the groups the services use", async () => {
  const puts = serve();
  await screen.findByRole("button", { name: "Переместить «Сводка»" });
  const now = document.querySelector('[data-section-block][data-section="now"]') as HTMLElement;
  await userEvent.selectOptions(within(now).getByLabelText("Тип виджета"), "services");
  await userEvent.click(within(now).getByRole("button", { name: "Добавить виджет" }));
  const added = order("now").at(-1) as string;
  await userEvent.click(within(tileOf(added)).getByRole("button", { name: "Настроить" }));
  const groups = await screen.findByLabelText(/^Группы/);
  await userEvent.click(groups);
  expect(screen.getAllByRole("option").map((option) => option.textContent)).toEqual(["Media", "Network"]);
  await userEvent.click(screen.getByRole("option", { name: "Network" }));
  await userEvent.click(groups);
  await userEvent.click(screen.getByRole("option", { name: "Media" }));
  await userEvent.click(screen.getByRole("button", { name: "Готово" }));
  await userEvent.click(screen.getByRole("button", { name: "Сохранить раскладку" }));
  await waitFor(() => expect(puts).toHaveLength(1));
  const sent = puts[0].body as { widgets: Array<{ type: string; settings: Record<string, unknown> }> };
  expect(sent.widgets.find((widget) => widget.type === "services")?.settings).toEqual({ groups: ["Network", "Media"] });
});
