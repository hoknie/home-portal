import { act, fireEvent, screen, waitFor, within } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { afterEach, beforeEach, expect, it, vi } from "vitest";

import { environmentKey, environmentSchema } from "@/entities/environment";
import { proxyKey, proxySchema } from "@/entities/proxy";
import { servicesSchema } from "@/entities/service";
import { apiSamples } from "@/shared/api";
import { jsonResponse, renderWithProviders, testQueryClient } from "@/shared/lib/testing";

import { ServiceForm } from "./service-form";

vi.mock("next/navigation", () => ({ useRouter: () => ({ push: vi.fn(), replace: vi.fn() }) }));

const nas = servicesSchema.parse(apiSamples.services).services[1];

beforeEach(() => {
  vi.stubGlobal("URL", Object.assign(URL, { createObjectURL: vi.fn(() => "blob:preview"), revokeObjectURL: vi.fn() }));
});

afterEach(() => {
  vi.unstubAllGlobals();
  vi.useRealTimers();
});

function seeded(enabled = true) {
  const client = testQueryClient();
  client.setQueryDefaults(proxyKey, { staleTime: Infinity });
  client.setQueryData(environmentKey, environmentSchema.parse(apiSamples.environment));
  client.setQueryData(proxyKey, { data: { ...proxySchema.parse(apiSamples.proxy), enabled }, revision: null });
  return client;
}

function open(onConflict = vi.fn(), service: typeof nas | null = nas, onSaved = vi.fn(), client = seeded()) {
  renderWithProviders(
    <ServiceForm service={service} environments={["local", "vpn"]} cancelHref="/service/?id=nas" revision='"r1"' taken={["media", "nas"]} groups={["Media", "Network"]} onSaved={onSaved} onConflict={onConflict} />,
    client,
  );
  return onConflict;
}

function add() {
  return open(vi.fn(), null);
}

it("shows the server's field errors next to the fields they name", async () => {
  vi.stubGlobal("fetch", vi.fn(async () => jsonResponse(apiSamples.fieldErrors, { status: 422 })));
  open();
  await userEvent.click(screen.getByRole("button", { name: "Save" }));
  expect(await screen.findByText("must start with a lower-case letter")).toBeInTheDocument();
  expect(screen.getByText("must use http or https")).toBeInTheDocument();
});

it("two people edit the same service: the second save conflicts, keeps what was typed and can overwrite", async () => {
  const fetch = vi.fn(async () => new Response("stale", { status: 409 }));
  vi.stubGlobal("fetch", fetch);
  const form = (revision: string) => <ServiceForm service={nas} environments={["local", "vpn"]} cancelHref="/service/?id=nas" revision={revision} taken={["media", "nas"]} groups={["Media", "Network"]} onSaved={vi.fn()} onConflict={vi.fn()} />;
  const view = renderWithProviders(form('"r1"'), seeded());
  view.rerender(form('"r2"'));
  const name = screen.getByLabelText("Name");
  await userEvent.clear(name);
  await userEvent.type(name, "Storage box");
  await userEvent.click(screen.getByRole("button", { name: "Save" }));
  expect(await screen.findByRole("button", { name: "Overwrite" })).toBeInTheDocument();
  expect(screen.getByRole("button", { name: "Reload" })).toBeInTheDocument();
  expect(screen.getByLabelText("Name")).toHaveValue("Storage box");
  const sent = (call: number) => ((fetch.mock.calls[call] as unknown as [string, RequestInit])[1].headers as Record<string, string>)["If-Match"];
  expect(sent(0)).toBe('"r1"');
  fetch.mockImplementation(async () => jsonResponse(apiSamples.services.services[1]));
  await userEvent.click(screen.getByRole("button", { name: "Overwrite" }));
  await waitFor(() => expect(fetch).toHaveBeenCalledTimes(2));
  expect(sent(1)).toBe('"r2"');
  expect(JSON.parse((fetch.mock.calls[1] as unknown as [string, RequestInit])[1].body as string).name).toBe("Storage box");
});

it("reloading after a conflict takes the current values once the person confirms", async () => {
  vi.stubGlobal("fetch", vi.fn(async () => new Response("stale", { status: 409 })));
  const form = (service: typeof nas, revision: string) => <ServiceForm service={service} environments={["local", "vpn"]} cancelHref="/service/?id=nas" revision={revision} taken={["media", "nas"]} groups={["Media", "Network"]} onSaved={vi.fn()} onConflict={vi.fn()} />;
  const view = renderWithProviders(form(nas, '"r1"'), seeded());
  await userEvent.clear(screen.getByLabelText("Name"));
  await userEvent.type(screen.getByLabelText("Name"), "Storage box");
  await userEvent.click(screen.getByRole("button", { name: "Save" }));
  await screen.findByRole("button", { name: "Reload" });
  view.rerender(form({ ...nas, name: "Changed by Boris" }, '"r2"'));
  await userEvent.click(screen.getByRole("button", { name: "Reload" }));
  await userEvent.click(within(screen.getByRole("dialog")).getByRole("button", { name: "Reload" }));
  expect(screen.getByLabelText("Name")).toHaveValue("Changed by Boris");
  expect(screen.queryByRole("button", { name: "Overwrite" })).not.toBeInTheDocument();
});

it("checks the fields on the client before sending", async () => {
  const fetch = vi.fn();
  vi.stubGlobal("fetch", fetch);
  open();
  const url = screen.getByLabelText("Address of Main address");
  await userEvent.clear(url);
  await userEvent.type(url, "ftp://nas");
  await userEvent.click(screen.getByRole("button", { name: "Save" }));
  expect(await screen.findByText("An absolute address is needed: http or https for an HTTP probe, any address with a host for TCP and ICMP")).toBeInTheDocument();
  expect(fetch).not.toHaveBeenCalled();
});

it("sends the revision it loaded", async () => {
  const fetch = vi.fn(async () => jsonResponse(apiSamples.services.services[1]));
  vi.stubGlobal("fetch", fetch);
  open();
  await userEvent.click(screen.getByRole("button", { name: "Save" }));
  await waitFor(() => expect(fetch).toHaveBeenCalled());
  const [path, init] = fetch.mock.calls[0] as unknown as [string, RequestInit];
  expect(path).toBe("/api/services/nas");
  expect((init.headers as Record<string, string>)["If-Match"]).toBe('"r1"');
});

it("switching to a tcp probe asks for a port and refuses an address without one", async () => {
  const fetch = vi.fn(async () => jsonResponse(apiSamples.services.services[1]));
  vi.stubGlobal("fetch", fetch);
  open(vi.fn(), { ...nas, proxy: null });
  await userEvent.selectOptions(screen.getByLabelText("Probe kind"), "tcp");
  expect(screen.getByLabelText(/^Port/)).toBeInTheDocument();
  expect(screen.queryByLabelText("Probe path")).not.toBeInTheDocument();
  const url = screen.getByLabelText("Address of Main address");
  await userEvent.clear(url);
  await userEvent.type(url, "tcp://printer.home.lan");
  await userEvent.click(screen.getByRole("button", { name: "Save" }));
  expect(await screen.findByText("Give a port from 1 to 65535, in the address or in the probe settings")).toBeInTheDocument();
  expect(fetch).not.toHaveBeenCalled();
  await userEvent.type(screen.getByLabelText(/^Port/), "9100");
  await userEvent.click(screen.getByRole("button", { name: "Save" }));
  await waitFor(() => expect(fetch).toHaveBeenCalled());
  const body = JSON.parse(String((fetch.mock.calls[0] as unknown as [string, RequestInit])[1].body));
  expect(body.probe).toMatchObject({ kind: "tcp", port: 9100 });
});

it("adds links and notes, checks link addresses and sends them", async () => {
  const fetch = vi.fn(async () => jsonResponse(apiSamples.services.services[1]));
  vi.stubGlobal("fetch", fetch);
  open();
  await userEvent.click(screen.getByRole("button", { name: "Add link" }));
  await userEvent.type(screen.getByLabelText("Link title"), "Admin");
  await userEvent.type(screen.getByLabelText("Link address"), "not a url");
  await userEvent.type(screen.getByLabelText(/^Notes/), "Restart with **docker**");
  await userEvent.click(screen.getByRole("button", { name: "Save" }));
  expect(await screen.findByText("An absolute http or https address is needed")).toBeInTheDocument();
  expect(fetch).not.toHaveBeenCalled();
  await userEvent.clear(screen.getByLabelText("Link address"));
  await userEvent.type(screen.getByLabelText("Link address"), "https://nas.local/admin");
  await userEvent.click(screen.getByRole("button", { name: "Save" }));
  await waitFor(() => expect(fetch).toHaveBeenCalled());
  const body = JSON.parse(String((fetch.mock.calls[0] as unknown as [string, RequestInit])[1].body));
  expect(body.links).toEqual([{ title: "Admin", url: "https://nas.local/admin" }]);
  expect(body.notes).toBe("Restart with **docker**");
});

it("shows a server error about a link beside that link", async () => {
  vi.stubGlobal("fetch", vi.fn(async () => jsonResponse({ errors: [{ field: "links[0].url", message: "must be an absolute http or https URL" }] }, { status: 422 })));
  open();
  await userEvent.click(screen.getByRole("button", { name: "Add link" }));
  await userEvent.type(screen.getByLabelText("Link title"), "Admin");
  await userEvent.type(screen.getByLabelText("Link address"), "https://nas.local/admin");
  await userEvent.click(screen.getByRole("button", { name: "Save" }));
  expect(await screen.findByText("must be an absolute http or https URL")).toBeInTheDocument();
});

it("proposes the id from a Russian name while adding", async () => {
  vi.stubGlobal("fetch", vi.fn());
  add();
  await userEvent.type(screen.getByLabelText("Name"), "Домашний NAS");
  expect(screen.getByLabelText("Identifier")).toHaveValue("domashniy-nas");
});

it("proposes a free id when the name's id is taken", async () => {
  vi.stubGlobal("fetch", vi.fn());
  add();
  await userEvent.type(screen.getByLabelText("Name"), "Media");
  expect(screen.getByLabelText("Identifier")).toHaveValue("media-2");
});

it("keeps an id typed by hand when the name changes", async () => {
  vi.stubGlobal("fetch", vi.fn());
  add();
  await userEvent.type(screen.getByLabelText("Identifier"), "nas");
  await userEvent.type(screen.getByLabelText("Name"), "Storage");
  expect(screen.getByLabelText("Identifier")).toHaveValue("nas");
});

it("leaves the id of an existing service alone when its name changes", async () => {
  vi.stubGlobal("fetch", vi.fn());
  open();
  const name = screen.getByLabelText("Name");
  await userEvent.clear(name);
  await userEvent.type(name, "Storage");
  expect(screen.getByLabelText("Identifier")).toHaveValue("nas");
});

it("chooses an existing group from the suggestions", async () => {
  const fetch = vi.fn(async () => jsonResponse(apiSamples.services.services[1]));
  vi.stubGlobal("fetch", fetch);
  open();
  await userEvent.type(screen.getByLabelText(/^Group/), "me");
  await userEvent.click(screen.getByRole("option", { name: "Media" }));
  await userEvent.click(screen.getByRole("button", { name: "Save" }));
  await waitFor(() => expect(fetch).toHaveBeenCalled());
  const body = JSON.parse(String((fetch.mock.calls[0] as unknown as [string, RequestInit])[1].body));
  expect(body.group).toBe("Media");
});

it("accepts a new group typed in and saves it", async () => {
  const fetch = vi.fn(async () => jsonResponse(apiSamples.services.services[1]));
  vi.stubGlobal("fetch", fetch);
  open();
  await userEvent.type(screen.getByLabelText(/^Group/), "Cameras{Enter}");
  await userEvent.click(screen.getByRole("button", { name: "Save" }));
  await waitFor(() => expect(fetch).toHaveBeenCalled());
  const body = JSON.parse(String((fetch.mock.calls[0] as unknown as [string, RequestInit])[1].body));
  expect(body.group).toBe("Cameras");
});

it("draws a lucide icon without asking the portal", async () => {
  const fetch = vi.fn();
  vi.stubGlobal("fetch", fetch);
  add();
  await userEvent.type(screen.getByLabelText(/^Icon/), "film");
  expect(document.querySelector('[data-icon-preview="film"]')).not.toBeNull();
  expect(fetch).not.toHaveBeenCalled();
});

it("asks the portal once for a fetched icon after a pause", async () => {
  vi.useFakeTimers({ shouldAdvanceTime: true });
  const fetch = vi.fn(async () => new Response(new Blob(["png"], { type: "image/png" }), { status: 200 }));
  vi.stubGlobal("fetch", fetch);
  add();
  fireEvent.change(screen.getByLabelText("Address of Main address"), { target: { value: "http://nas.local" } });
  fireEvent.change(screen.getByLabelText(/^Icon/), { target: { value: "catalog:jellyfin" } });
  expect(fetch).not.toHaveBeenCalled();
  await act(async () => {
    await vi.advanceTimersByTimeAsync(600);
  });
  await waitFor(() => expect(document.querySelector('[data-icon-preview="image"]')).not.toBeNull());
  expect(fetch).toHaveBeenCalledOnce();
  const [path, init] = fetch.mock.calls[0] as unknown as [string, RequestInit];
  expect(path).toBe("/api/icon-preview");
  expect(JSON.parse(String(init.body))).toEqual({ icon: "catalog:jellyfin", url: null });
});

it("shows why a preview failed and still saves", async () => {
  vi.useFakeTimers({ shouldAdvanceTime: true });
  const fetch = vi.fn(async (path: string) =>
    path === "/api/icon-preview"
      ? jsonResponse({ errors: [{ field: "icon", message: "what was fetched is not an image the portal serves" }] }, { status: 422 })
      : jsonResponse(apiSamples.services.services[1]),
  );
  vi.stubGlobal("fetch", fetch);
  open();
  const icon = screen.getByLabelText(/^Icon/);
  fireEvent.change(icon, { target: { value: "url:http://nas.local/" } });
  fireEvent.blur(icon);
  expect(await screen.findByText("what was fetched is not an image the portal serves")).toBeInTheDocument();
  expect(document.querySelector('[data-icon-preview="default"]')).not.toBeNull();
  fireEvent.click(screen.getByRole("button", { name: "Save" }));
  await waitFor(() => expect(fetch.mock.calls.some(([path]) => path === "/api/services/nas")).toBe(true));
});

it("aborts the previous preview when the value changes again", async () => {
  vi.useFakeTimers({ shouldAdvanceTime: true });
  const signals: AbortSignal[] = [];
  const fetch = vi.fn((_: string, init: RequestInit) => {
    signals.push(init.signal as AbortSignal);
    return new Promise<Response>(() => {});
  });
  vi.stubGlobal("fetch", fetch);
  add();
  const icon = screen.getByLabelText(/^Icon/);
  fireEvent.change(icon, { target: { value: "catalog:jellyfin" } });
  fireEvent.blur(icon);
  await waitFor(() => expect(signals).toHaveLength(1));
  fireEvent.change(icon, { target: { value: "catalog:plex" } });
  fireEvent.blur(icon);
  await waitFor(() => expect(signals).toHaveLength(2));
  expect(signals[0].aborted).toBe(true);
});

it("a host already taken is reported next to the host field", async () => {
  vi.stubGlobal(
    "fetch",
    vi.fn(async () =>
      jsonResponse({ errors: [{ field: "proxy.host", message: "is published by another service" }] }, { status: 422 }),
    ),
  );
  open();
  await userEvent.click(screen.getByRole("button", { name: "Save" }));
  const published = screen.getByRole("listitem", { name: "Address 2" });
  expect(await within(published).findByText("is published by another service")).toBeInTheDocument();
});

it("an error about another part of the configuration is shown above the form", async () => {
  vi.stubGlobal(
    "fetch",
    vi.fn(async () =>
      jsonResponse({ errors: [{ field: "services[2].proxy.auth", message: "must be under proxy.cookie_domain" }] }, { status: 422 }),
    ),
  );
  open();
  await userEvent.click(screen.getByRole("button", { name: "Save" }));
  const notice = await screen.findByText("The service was not saved: the configuration has other problems");
  expect(notice.closest("[role=alert]")).toHaveTextContent("services[2].proxy.auth: must be under proxy.cookie_domain");
});

it("files mode asks for both files before sending", async () => {
  const fetch = vi.fn();
  vi.stubGlobal("fetch", fetch);
  open();
  await userEvent.selectOptions(screen.getByLabelText("Certificate"), "files");
  await userEvent.click(screen.getByRole("button", { name: "Save" }));
  expect(await screen.findAllByText("A file path is needed")).toHaveLength(2);
  expect(fetch).not.toHaveBeenCalled();
});


it("shows every setting of an http probe without a click", () => {
  open(vi.fn(), { ...nas, proxy: null });
  for (const label of ["Probe kind", "Probe path", "Interval, s", "Timeout, s", "Slow after, ms"]) {
    expect(screen.getByLabelText(label)).toBeVisible();
  }
  expect(document.querySelector("details, summary")).toBeNull();
});

it("two links share one header row and each input keeps its own label", () => {
  open(vi.fn(), { ...nas, links: [{ title: "Admin", url: "http://nas.local/admin" }, { title: "Logs", url: "http://nas.local/logs" }] });
  const block = screen.getByRole("group", { name: "Links" });
  expect(within(block).getAllByText("Title")).toHaveLength(1);
  expect(within(block).getAllByText("Address")).toHaveLength(1);
  expect(screen.getAllByLabelText("Link title").map((input) => (input as HTMLInputElement).value)).toEqual(["Admin", "Logs"]);
  expect(screen.getAllByLabelText("Link address")).toHaveLength(2);
});
