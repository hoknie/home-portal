import { screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { afterEach, expect, it, vi } from "vitest";

import { type Modules, modulesSchema } from "@/entities/module";
import { apiSamples } from "@/shared/api";
import { jsonResponse, renderWithProviders } from "@/shared/lib/testing";
import { Toaster } from "@/shared/ui/primitives";

import { ModuleSwitch } from "./module-switch";

const sample = modulesSchema.parse(apiSamples.modules);

function withModule(name: string, change: Partial<Modules["modules"][number]>): Modules {
  return { modules: sample.modules.map((module) => (module.name === name ? { ...module, ...change } : module)) };
}

function show(modules: Modules, name: string) {
  const shown = modules.modules.find((entry) => entry.name === name)!;
  renderWithProviders(
    <>
      <ModuleSwitch module={shown} modules={modules} revision={'"r1"'} />
      <Toaster />
    </>,
  );
}

afterEach(() => {
  vi.unstubAllGlobals();
});

it("a free module switches with the loaded revision and says so", async () => {
  const fetch = vi.fn(async () => jsonResponse(withModule("webhooks", { enabled: true }), { headers: { ETag: '"r2"' } }));
  vi.stubGlobal("fetch", fetch);
  show(sample, "webhooks");
  await userEvent.click(screen.getByRole("switch", { name: "Webhooks: on or off" }));
  expect(await screen.findByText("Webhooks is on")).toBeInTheDocument();
  const [path, init] = fetch.mock.calls[0] as unknown as [string, RequestInit];
  expect(path).toBe("/api/modules/webhooks");
  expect(init.method).toBe("PUT");
  expect((init.headers as Record<string, string>)["If-Match"]).toBe('"r1"');
  expect(init.body).toBe('{"enabled":true}');
});

it("a module another one needs is locked and says which to switch off first", () => {
  show(sample, "proxy");
  expect(screen.getByRole("switch", { name: "Proxy: on or off" })).toBeDisabled();
  expect(screen.getByText("Switch DNS off first")).toBeInTheDocument();
});

it("a module whose requirement is off is locked and says which to switch on first", () => {
  const off = withModule("automations", { enabled: false });
  show(off, "webhooks");
  expect(screen.getByRole("switch", { name: "Webhooks: on or off" })).toBeDisabled();
  expect(screen.getByText("Switch Automations on first")).toBeInTheDocument();
});

it("incomplete settings list the missing fields and link to the module's page", async () => {
  vi.stubGlobal(
    "fetch",
    vi.fn(async () => jsonResponse({ errors: [{ field: "proxy.portal_host", message: "is required" }] }, { status: 422 })),
  );
  const allOff: Modules = {
    modules: sample.modules.map((module) =>
      module.name === "proxy" || module.name === "dns" ? { ...module, enabled: false, required_by: [] } : module,
    ),
  };
  show(allOff, "proxy");
  await userEvent.click(screen.getByRole("switch", { name: "Proxy: on or off" }));
  expect(await screen.findByText("proxy.portal_host")).toBeInTheDocument();
  expect(screen.getByRole("link", { name: "Configure" })).toHaveAttribute("href", "/admin/proxy");
});

it("a refusal from the server is shown as a toast", async () => {
  vi.stubGlobal("fetch", vi.fn(async () => new Response("webhooks needs automations; switch webhooks off first", { status: 409 })));
  show(withModule("webhooks", { enabled: false }), "automations");
  await userEvent.click(screen.getByRole("switch", { name: "Automations: on or off" }));
  expect(await screen.findByText(/Automations was not switched: webhooks needs automations/)).toBeInTheDocument();
});
