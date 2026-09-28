import { screen, waitFor, within } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { afterEach, expect, it, vi } from "vitest";

import { type Notifications, channelSchema, notificationsKey, notificationsSchema } from "@/entities/notification";
import { apiSamples } from "@/shared/api";
import { jsonResponse, renderWithProviders, testQueryClient } from "@/shared/lib/testing";

import { NotificationsScreen } from "./notifications-screen";

afterEach(() => {
  vi.unstubAllGlobals();
});

const ready = notificationsSchema.parse(apiSamples.notifications);
const missing: Notifications = { ...ready, channels: [channelSchema.parse(apiSamples.notificationChannelMissing)] };

function render(notifications: Notifications, answer: (path: string, init?: RequestInit) => Response) {
  const fetch = vi.fn(async (path: string, init?: RequestInit) =>
    path === "/api/secrets" ? jsonResponse({ secrets: [{ name: "telegram_token", set: false }, { name: "bot", set: true }] }) : answer(path, init),
  );
  vi.stubGlobal("fetch", fetch);
  const client = testQueryClient();
  client.setQueryDefaults(notificationsKey, { staleTime: Infinity });
  client.setQueryData(notificationsKey, { data: notifications, revision: '"n1"' });
  renderWithProviders(<NotificationsScreen />, client);
  return fetch;
}

function card() {
  return screen.getByRole("form", { name: "Channel settings" }).closest("[data-slot=card]") as HTMLElement;
}

it("a missing token names the key, and the test cannot be sent", () => {
  render(missing, () => jsonResponse({}));
  expect(within(card()).getByText("Not set up")).toBeInTheDocument();
  expect(within(card()).getByText("notifications.telegram.secret: names telegram_token, which is not set")).toBeInTheDocument();
  expect(within(card()).getByText("3 waiting · 0 dropped")).toBeInTheDocument();
  expect(within(card()).getByRole("button", { name: "Send test" })).toBeDisabled();
});

it("setting up Telegram saves the chosen secret with the revision and a server refusal shows beside its field", async () => {
  let attempt = 0;
  const fetch = render(missing, (_path, init) => {
    if (init?.method !== "PUT") {
      return jsonResponse(ready);
    }
    attempt += 1;
    return attempt === 1 ? jsonResponse({ errors: [{ field: "chat_id", message: "must be a number" }] }, { status: 422 }) : jsonResponse(ready, { headers: { ETag: '"n2"' } });
  });
  const secret = await within(card()).findByRole("option", { name: "bot" });
  await userEvent.selectOptions(secret.closest("select")!, "bot");
  await userEvent.click(within(card()).getByRole("button", { name: "Save channel" }));
  expect(await within(card()).findByText("must be a number")).toBeInTheDocument();
  await userEvent.click(within(card()).getByRole("button", { name: "Save channel" }));
  await waitFor(() => expect(attempt).toBe(2));
  const [path, init] = fetch.mock.calls.find(([, options]) => options?.method === "PUT") as unknown as [string, RequestInit];
  expect(path).toBe("/api/notifications/channels/telegram");
  expect((init.headers as Record<string, string>)["If-Match"]).toBe('"n1"');
  expect(JSON.parse(String(init.body))).toEqual({ chat_id: "123456789", enabled: true, secret: "bot" });
});

it("a ready channel sends a test and says whether it was delivered", async () => {
  const fetch = render(ready, (path) => (path === "/api/notifications/test" ? jsonResponse(apiSamples.notificationTest) : jsonResponse(ready)));
  expect(within(card()).getByText("Ready")).toBeInTheDocument();
  expect(within(card()).getByText("telegram answered 502 Bad Gateway")).toBeInTheDocument();
  await userEvent.click(within(card()).getByRole("button", { name: "Send test" }));
  expect(await within(card()).findByText("The test message was delivered.")).toBeInTheDocument();
  expect(fetch).toHaveBeenCalledWith("/api/notifications/test", expect.objectContaining({ method: "POST", body: JSON.stringify({ channel: "telegram" }) }));
});

it("the rules save the chosen states and recoveries", async () => {
  const fetch = render(ready, () => jsonResponse(ready));
  await userEvent.click(screen.getByRole("checkbox", { name: "Slow" }));
  await userEvent.click(screen.getByRole("switch", { name: "Announce when a service is up again" }));
  await userEvent.click(screen.getByRole("button", { name: "Save rules" }));
  await waitFor(() => expect(fetch).toHaveBeenCalledWith("/api/notifications", expect.objectContaining({ method: "PUT" })));
  const [, init] = fetch.mock.calls.find(([path, options]) => path === "/api/notifications" && options?.method === "PUT") as unknown as [string, RequestInit];
  expect(JSON.parse(String(init.body))).toEqual({ states: ["down", "unreadable", "degraded"], recovered: false });
});

it("while the module is off the page says so and keeps the settings editable", () => {
  render({ ...ready, enabled: false }, () => jsonResponse({}));
  expect(screen.getByRole("status")).toHaveTextContent("The Notifications module is off");
  expect(within(card()).getByRole("button", { name: "Send test" })).toBeDisabled();
  expect(within(card()).getByRole("button", { name: "Save channel" })).toBeEnabled();
});
