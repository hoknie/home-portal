import { fireEvent, screen, waitFor } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { afterEach, expect, it, vi } from "vitest";

import { servicesKey, servicesSchema } from "@/entities/service";
import { apiSamples } from "@/shared/api";
import { jsonResponse, renderWithProviders, testQueryClient } from "@/shared/lib/testing";

import { ServiceEditorScreen } from "./service-editor-screen";

let search = "";
const push = vi.fn();

vi.mock("next/navigation", () => ({
  useSearchParams: () => new URLSearchParams(search),
  usePathname: () => "/admin/services/edit/",
  useRouter: () => ({ push, replace: vi.fn() }),
}));

afterEach(() => {
  search = "";
  push.mockReset();
  vi.unstubAllGlobals();
  vi.restoreAllMocks();
});

function renderScreen(mode: "new" | "edit") {
  const client = testQueryClient();
  client.setQueryData(servicesKey, { data: servicesSchema.parse(apiSamples.services), revision: '"r"' });
  return renderWithProviders(<ServiceEditorScreen mode={mode} />, client);
}

it("adds a service on its own page and returns to the list after saving", async () => {
  const fetch = vi.fn(async () => jsonResponse(apiSamples.services.services[1], { status: 201 }));
  vi.stubGlobal("fetch", fetch);
  renderScreen("new");
  expect(screen.getByRole("heading", { level: 1, name: "New service" })).toBeInTheDocument();
  expect(screen.getByLabelText("Name")).toHaveValue("");
  await userEvent.type(screen.getByLabelText("Name"), "Scanner");
  await userEvent.type(screen.getByLabelText(/^Address$/), "http://scanner.local");
  await userEvent.click(screen.getByRole("button", { name: "Save" }));
  await waitFor(() => expect(push).toHaveBeenCalledWith("/admin/services/"));
  const post = (fetch.mock.calls as unknown as [string, RequestInit | undefined][]).find(([, init]) => init?.method === "POST");
  const body = JSON.parse(String(post?.[1]?.body));
  expect(body.id).toBe("scanner");
});

it("edits the service named in the address", () => {
  search = "id=nas";
  renderScreen("edit");
  expect(screen.getByRole("heading", { level: 1, name: "Edit service" })).toBeInTheDocument();
  expect(screen.getByLabelText("Identifier")).toHaveValue("nas");
});

it("says an unknown service was not found and links back to the list", () => {
  search = "id=nope";
  renderScreen("edit");
  expect(screen.getByText("Service not found")).toBeInTheDocument();
  expect(screen.getByRole("link", { name: "Back to the services" }).getAttribute("href")).toMatch(/^\/admin\/services\/?$/);
});

it("asks before leaving with unsaved changes", async () => {
  search = "id=nas";
  const confirm = vi.spyOn(window, "confirm").mockReturnValue(false);
  renderScreen("edit");
  const cancel = screen.getByRole("link", { name: "Cancel" });
  fireEvent.click(cancel);
  expect(confirm).not.toHaveBeenCalled();
  await userEvent.type(screen.getByLabelText("Name"), " box");
  fireEvent.click(cancel);
  expect(confirm).toHaveBeenCalledWith("The service has unsaved changes. Leave the page?");
});
