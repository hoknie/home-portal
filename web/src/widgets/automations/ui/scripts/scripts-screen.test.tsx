import { screen, waitFor, within } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { afterEach, expect, it, vi } from "vitest";

import { scriptTextSchema, scriptTreeSchema } from "@/entities/script";
import { apiSamples } from "@/shared/api";
import { jsonResponse, renderWithProviders } from "@/shared/lib/testing";

import { ScriptsScreen } from "./scripts-screen";

vi.mock("next/navigation", () => ({
  useRouter: () => ({ push: vi.fn(), replace: vi.fn() }),
  usePathname: () => "/admin/scripts/",
}));

afterEach(() => {
  vi.unstubAllGlobals();
});

const tree = scriptTreeSchema.parse(apiSamples.scriptsTree);
const text = scriptTextSchema.parse(apiSamples.scriptText);
const nightly = {
  ...tree.files[0],
  path: "backup/nightly.sh",
  folder: "backup",
  name: "nightly.sh",
};

type Answer = (path: string, init?: RequestInit) => Response | undefined;

function serve({ editing = true, inside = true, answer = (() => undefined) as Answer, files = tree.files, folders = tree.folders } = {}) {
  const fetch = vi.fn(async (path: string, init?: RequestInit) => {
    const own = answer(path, init);
    if (own) {
      return own;
    }
    if (path === "/api/automations/scripts") {
      return jsonResponse({ ...apiSamples.automationScripts, editing });
    }
    if (path === "/api/scripts") {
      return jsonResponse({
        ...apiSamples.scriptsTree,
        inside,
        files,
        folders,
      });
    }
    if (path.startsWith("/api/scripts/file?path=")) {
      return jsonResponse(apiSamples.scriptText, {
        headers: { ETag: `"${text.revision}"` },
      });
    }
    if (path === "/api/automations") {
      return jsonResponse(apiSamples.automations);
    }
    if (path === "/api/webhooks") {
      return jsonResponse(apiSamples.webhooks);
    }
    if (path === "/api/workflows") {
      return jsonResponse(apiSamples.workflows);
    }
    return jsonResponse({});
  });
  vi.stubGlobal("fetch", fetch);
  renderWithProviders(<ScriptsScreen />);
  return fetch;
}

function sent(fetch: ReturnType<typeof serve>, method: string, path: string) {
  const call = fetch.mock.calls.find(([called, init]) => called === path && (init as RequestInit | undefined)?.method === method);
  return call ? JSON.parse(String((call[1] as RequestInit).body)) : undefined;
}

it("says how to switch editing on while the file keeps it off", async () => {
  serve({ editing: false });
  expect(await screen.findByText("[scripts] editing = true")).toBeInTheDocument();
  expect(screen.queryByRole("navigation", { name: "Files" })).not.toBeInTheDocument();
});

it("creates a script in a folder, then saves its header and shows the argument it declares", async () => {
  const fetch = serve({
    answer: (path, init) =>
      (init?.method === "POST" || init?.method === "PUT") && path.startsWith("/api/scripts/file")
        ? jsonResponse(text.entry, {
            status: init.method === "POST" ? 201 : 200,
            headers: { ETag: '"r2"' },
          })
        : undefined,
  });
  await userEvent.click(await screen.findByRole("button", { name: "New script" }));
  const dialog = await screen.findByRole("dialog");
  await userEvent.selectOptions(within(dialog).getByLabelText("Folder"), "media");
  await userEvent.type(within(dialog).getByLabelText("Name"), "restart.sh");
  await userEvent.click(within(dialog).getByRole("button", { name: "Create" }));
  await waitFor(() =>
    expect(sent(fetch, "POST", "/api/scripts/file")).toMatchObject({
      path: "media/restart.sh",
    }),
  );
  expect(sent(fetch, "POST", "/api/scripts/file").content).toContain("# @arg service <text>");
  const area = await screen.findByRole("textbox", { name: "Script text" });
  await userEvent.type(area, "# @arg --dry-run");
  await userEvent.click(screen.getByRole("button", { name: "Save" }));
  await waitFor(() => expect(sent(fetch, "PUT", "/api/scripts/file?path=media%2Frestart.sh")).toBeDefined());
  const declared = screen.getByRole("region", {
    name: "Declared by the header",
  });
  expect(within(declared).getByText("service")).toBeInTheDocument();
  expect(within(screen.getByRole("navigation", { name: "Files" })).getByText("restart.sh")).toBeInTheDocument();
});

it("the confirmation of a delete names the automations that use the script", async () => {
  serve({ files: [nightly], folders: ["backup"] });
  (await screen.findByRole("button", { name: /backup\/nightly\.sh: Rename or move/ })).focus();
  await userEvent.keyboard("{Enter}");
  await userEvent.click(await screen.findByRole("menuitem", { name: "Delete" }));
  const dialog = await screen.findByRole("dialog");
  await waitFor(() => expect(within(dialog).getByText(/Nightly backup/)).toBeInTheDocument());
});

it("from outside the page shows the files and text but offers no change", async () => {
  serve({ inside: false });
  expect(await screen.findByText("Scripts can be changed only from inside your environments, so this page is read-only here.")).toBeInTheDocument();
  await userEvent.click(within(screen.getByRole("navigation", { name: "Files" })).getByText("restart.sh"));
  expect(await screen.findByRole("textbox", { name: "Script text" })).toHaveAttribute("readonly");
  for (const name of ["Save", "New script", "New folder"]) {
    expect(screen.queryByRole("button", { name })).not.toBeInTheDocument();
  }
  expect(screen.queryByRole("button", { name: /Rename or move/ })).not.toBeInTheDocument();
});

it("an open script shows how it is called, its arguments as cards, and a header help that inserts an example", async () => {
  serve();
  await userEvent.click(await within(await screen.findByRole("navigation", { name: "Files" })).findByText("restart.sh"));
  const declared = await screen.findByRole("region", { name: "Declared by the header" });
  expect(within(declared).getByTestId("script-usage")).toHaveTextContent("restart.sh <service> [--force]");
  expect(within(declared).getByText("Restart a service's container")).toBeInTheDocument();
  expect(within(declared).getAllByText("required")).toHaveLength(1);
  expect(within(declared).getByText("switch")).toBeInTheDocument();
  await userEvent.click(screen.getByText("Header format"));
  await userEvent.click(screen.getByRole("button", { name: "Insert an example header" }));
  const area = screen.getByRole("textbox", { name: "Script text" }) as HTMLTextAreaElement;
  expect(area.value.split("\n")[1]).toBe("# @description Restart a service's container");
  expect(screen.getByText("Unsaved changes")).toBeInTheDocument();
});
