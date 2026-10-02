import { act, screen, waitFor, within } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { afterEach, expect, it, vi } from "vitest";

import { sessionKey } from "@/entities/session";
import { type Users, groupsKey, groupsSchema, usersKey, usersSchema } from "@/entities/user";
import { apiSamples } from "@/shared/api";
import { jsonResponse, renderWithProviders, testQueryClient } from "@/shared/lib/testing";
import { Toaster } from "@/shared/ui/kit";

import { UsersScreen } from "./users-screen";

function renderWith(change: (users: Users) => void = () => undefined, rights: Record<string, string[]> | null = null) {
  const users = usersSchema.parse(structuredClone(apiSamples.users));
  change(users);
  const client = testQueryClient();
  if (rights !== null) {
    client.setQueryData(sessionKey, { name: "anna", group: "family", admin: false, rights });
    users.users = users.users.map((user) => ({ ...user, you: user.name === "anna" }));
  }
  client.setQueryDefaults(usersKey, { staleTime: Infinity });
  client.setQueryData(usersKey, { data: users, revision: '"r1"' });
  client.setQueryDefaults(groupsKey, { staleTime: Infinity });
  client.setQueryData(groupsKey, { data: groupsSchema.parse(apiSamples.groups), revision: '"g1"' });
  renderWithProviders(
    <>
      <UsersScreen />
      <Toaster />
    </>,
    client,
  );
  return client;
}

function row(name: string) {
  return screen.getAllByRole("cell", { name: new RegExp(`^${name}`) })[0].closest("tr") as HTMLElement;
}

afterEach(() => {
  vi.unstubAllGlobals();
});

it("adding a user sends the name and password with the loaded revision and lists them", async () => {
  const answered = usersSchema.parse(structuredClone(apiSamples.users));
  answered.users.push({ name: "bob", group: null, you: false });
  const fetch = vi.fn(async () => jsonResponse(answered, { status: 201, headers: { ETag: '"r2"' } }));
  vi.stubGlobal("fetch", fetch);
  renderWith();
  await userEvent.click(screen.getByRole("button", { name: "Add user" }));
  const dialog = await screen.findByRole("dialog");
  await userEvent.type(within(dialog).getByLabelText("Name"), "bob");
  await userEvent.type(within(dialog).getByLabelText(/^Password/), "correct horse");
  await userEvent.type(within(dialog).getByLabelText("Repeat the password"), "correct horse");
  await userEvent.click(within(dialog).getByRole("button", { name: "Add" }));
  await waitFor(() => expect(fetch).toHaveBeenCalled());
  const [path, init] = fetch.mock.calls[0] as unknown as [string, RequestInit];
  expect(path).toBe("/api/users");
  expect(init.method).toBe("POST");
  expect((init.headers as Record<string, string>)["If-Match"]).toBe('"r1"');
  expect(JSON.parse(String(init.body))).toEqual({ name: "bob", password: "correct horse", group: null });
  expect(await screen.findByText("bob can now sign in")).toBeInTheDocument();
  expect(await screen.findByRole("cell", { name: "bob" })).toBeInTheDocument();
});

it("passwords that do not match are shown beside the second field and nothing is sent", async () => {
  const fetch = vi.fn();
  vi.stubGlobal("fetch", fetch);
  renderWith();
  await userEvent.click(screen.getByRole("button", { name: "Add user" }));
  const dialog = await screen.findByRole("dialog");
  await userEvent.type(within(dialog).getByLabelText("Name"), "bob");
  await userEvent.type(within(dialog).getByLabelText(/^Password/), "correct horse");
  await userEvent.type(within(dialog).getByLabelText("Repeat the password"), "correct hors");
  await userEvent.click(within(dialog).getByRole("button", { name: "Add" }));
  expect(await within(dialog).findByText("The passwords do not match")).toBeInTheDocument();
  expect(fetch).not.toHaveBeenCalled();
});

it("a server refusal of the name is shown beside the name", async () => {
  vi.stubGlobal("fetch", vi.fn(async () => jsonResponse({ errors: [{ field: "name", message: "is used by another user" }] }, { status: 422 })));
  renderWith();
  await userEvent.click(screen.getByRole("button", { name: "Add user" }));
  const dialog = await screen.findByRole("dialog");
  await userEvent.type(within(dialog).getByLabelText("Name"), "anna");
  await userEvent.type(within(dialog).getByLabelText(/^Password/), "correct horse");
  await userEvent.type(within(dialog).getByLabelText("Repeat the password"), "correct horse");
  await userEvent.click(within(dialog).getByRole("button", { name: "Add" }));
  expect(await within(dialog).findByText("is used by another user")).toBeInTheDocument();
});

it("one's own row is marked you and cannot be deleted", () => {
  renderWith();
  const own = row("admin");
  expect(within(own).getByText("you")).toBeInTheDocument();
  const remove = within(own).getByRole("button", { name: "Delete" });
  expect(remove).toBeDisabled();
  expect(remove).toHaveAttribute("title", "You cannot delete yourself");
  expect(within(row("anna")).getByRole("button", { name: "Delete" })).toBeEnabled();
});

it("users while off: the list stays, the notice says so, and nothing can be changed", () => {
  renderWith((users) => {
    users.editable = false;
  });
  expect(screen.getByRole("status")).toHaveTextContent("The Users module is off");
  expect(screen.getAllByRole("cell", { name: "anna" })[0]).toBeInTheDocument();
  expect(screen.getByRole("button", { name: "Add user" })).toBeDisabled();
  for (const button of screen.getAllByRole("button", { name: "Change password" })) {
    expect(button).toBeDisabled();
  }
  for (const button of screen.getAllByRole("button", { name: "Delete" })) {
    expect(button).toBeDisabled();
  }
});

it("changing a password sends it to that user's address", async () => {
  const fetch = vi.fn(async () => jsonResponse(apiSamples.users, { headers: { ETag: '"r2"' } }));
  vi.stubGlobal("fetch", fetch);
  renderWith();
  await userEvent.click(within(row("anna")).getByRole("button", { name: "Change password" }));
  const dialog = await screen.findByRole("dialog");
  await userEvent.type(within(dialog).getByLabelText(/^Password/), "a brand new one");
  await userEvent.type(within(dialog).getByLabelText("Repeat the password"), "a brand new one");
  await userEvent.click(within(dialog).getByRole("button", { name: "Save password" }));
  await waitFor(() => expect(fetch).toHaveBeenCalled());
  const [path, init] = fetch.mock.calls[0] as unknown as [string, RequestInit];
  expect(path).toBe("/api/users/anna/password");
  expect(JSON.parse(String(init.body))).toEqual({ password: "a brand new one" });
  expect(await screen.findByText("The password of anna was changed")).toBeInTheDocument();
});

it("each user shows their group, and a user without one says so", () => {
  renderWith();
  expect(within(row("anna")).getByText("family")).toBeInTheDocument();
  expect(within(row("guest")).getByText("No group")).toBeInTheDocument();
});

it("editing a group sends its new rights from the matrix", async () => {
  const fetch = vi.fn(async () => jsonResponse(apiSamples.groups, { headers: { ETag: '"g2"' } }));
  vi.stubGlobal("fetch", fetch);
  renderWith();
  await userEvent.click(screen.getByRole("tab", { name: "Groups" }));
  const family = screen.getAllByRole("cell", { name: /^family/ })[0].closest("tr") as HTMLElement;
  await userEvent.click(within(family).getByRole("button", { name: "Edit" }));
  const dialog = await screen.findByRole("dialog");
  await userEvent.click(within(dialog).getByRole("checkbox", { name: "Workflows: Run" }));
  await userEvent.click(within(dialog).getByRole("button", { name: "Save" }));
  await waitFor(() => expect(fetch).toHaveBeenCalled());
  const [path, init] = fetch.mock.calls[0] as unknown as [string, RequestInit];
  expect(path).toBe("/api/groups/family");
  expect(init.method).toBe("PUT");
  expect(JSON.parse(String(init.body))).toEqual({ name: "family", rights: { automations: ["read", "execute"], services: ["update"], workflows: ["execute"] } });
});

it("a users manager outside admin cannot touch admins, give more than they hold, or change groups", async () => {
  renderWith(() => undefined, { users: ["read", "create", "update", "delete"], automations: ["read", "execute"], services: ["update"] });
  expect(within(row("admin")).queryByRole("button")).toBeNull();
  await userEvent.click(within(row("guest")).getByRole("button", { name: "Change group" }));
  const dialog = await screen.findByRole("dialog");
  const options = within(dialog).getAllByRole("option").map((option) => option.textContent);
  expect(options).toEqual(["No group", "family", "guests"]);
  await userEvent.keyboard("{Escape}");
  await userEvent.click(screen.getByRole("tab", { name: "Groups" }));
  expect(screen.queryByRole("button", { name: "Add group" })).toBeNull();
  expect(screen.queryByRole("button", { name: "Edit" })).toBeNull();
});

it("an admin sees admin with every right and cannot edit it", async () => {
  renderWith();
  await userEvent.click(screen.getByRole("tab", { name: "Groups" }));
  expect(screen.getByRole("button", { name: "Add group" })).toBeInTheDocument();
  const boxes = screen.getAllByRole("checkbox");
  expect(boxes.length).toBeGreaterThan(0);
  expect(boxes.every((box) => box.getAttribute("aria-checked") === "true" && box.hasAttribute("disabled"))).toBe(true);
});

it("changing one's own password asks for the current one and sends it", async () => {
  const fetch = vi.fn(async () => jsonResponse(apiSamples.users));
  vi.stubGlobal("fetch", fetch);
  renderWith();
  await userEvent.click(within(row("admin")).getByRole("button", { name: "Change password" }));
  const dialog = await screen.findByRole("dialog");
  await userEvent.type(within(dialog).getByLabelText(/^Password/), "a brand new one");
  await userEvent.type(within(dialog).getByLabelText("Repeat the password"), "a brand new one");
  await userEvent.click(within(dialog).getByRole("button", { name: "Save password" }));
  expect(await within(dialog).findByText("Type your current password")).toBeInTheDocument();
  expect(fetch).not.toHaveBeenCalled();
  await userEvent.type(within(dialog).getByLabelText("Current password"), "secret99");
  await userEvent.click(within(dialog).getByRole("button", { name: "Save password" }));
  await waitFor(() => expect(fetch).toHaveBeenCalled());
  const [, init] = fetch.mock.calls[0] as unknown as [string, RequestInit];
  expect(JSON.parse(String(init.body))).toEqual({ password: "a brand new one", current_password: "secret99" });
});

it("changing another user's password does not ask for a current one", async () => {
  renderWith();
  await userEvent.click(within(row("anna")).getByRole("button", { name: "Change password" }));
  const dialog = await screen.findByRole("dialog");
  expect(within(dialog).queryByLabelText("Current password")).toBeNull();
});

it("a dialog sends the revision it was opened with, and a conflict can be overwritten", async () => {
  const puts = () => fetch.mock.calls.filter(([, init]) => init?.method === "PUT");
  const fetch = vi.fn(async (_path: string, init?: RequestInit) =>
    init?.method === "PUT" && puts().length === 1 ? new Response("stale", { status: 409 }) : jsonResponse(apiSamples.users, { headers: { ETag: '"r3"' } }),
  );
  vi.stubGlobal("fetch", fetch);
  const client = renderWith();
  await userEvent.click(within(row("guest")).getByRole("button", { name: "Change group" }));
  act(() => client.setQueryData(usersKey, { data: usersSchema.parse(apiSamples.users), revision: '"r2"' }));
  await userEvent.click(screen.getByRole("button", { name: "Save group" }));
  await userEvent.click(await screen.findByRole("button", { name: "Overwrite" }));
  await waitFor(() => expect(puts()).toHaveLength(2));
  expect(puts().map(([, init]) => (init?.headers as Record<string, string>)["If-Match"])).toEqual(['"r1"', '"r3"']);
});

it("a group given everything with one press is saved with every right of the matrix", async () => {
  const fetch = vi.fn(async () => jsonResponse(apiSamples.groups, { headers: { ETag: '"g2"' } }));
  vi.stubGlobal("fetch", fetch);
  renderWith();
  await userEvent.click(screen.getByRole("tab", { name: "Groups" }));
  const family = screen.getAllByRole("cell", { name: /^family/ })[0].closest("tr") as HTMLElement;
  await userEvent.click(within(family).getByRole("button", { name: "Edit" }));
  const dialog = await screen.findByRole("dialog");
  await userEvent.click(within(dialog).getByRole("button", { name: "Select all" }));
  await userEvent.click(within(dialog).getByRole("button", { name: "Save" }));
  await waitFor(() => expect(fetch).toHaveBeenCalled());
  const matrix = groupsSchema.parse(apiSamples.groups).matrix;
  const sent = JSON.parse(String((fetch.mock.calls[0] as unknown as [string, RequestInit])[1].body));
  expect(sent.rights).toEqual(Object.fromEntries(matrix.map((row) => [row.area, row.actions])));
});
