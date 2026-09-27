import { screen, waitFor, within } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { afterEach, expect, it, vi } from "vitest";

import { type Users, usersKey, usersSchema } from "@/entities/user";
import { apiSamples } from "@/shared/api";
import { jsonResponse, renderWithProviders, testQueryClient } from "@/shared/lib/testing";
import { Toaster } from "@/shared/ui/primitives";

import { UsersScreen } from "./users-screen";

function renderWith(change: (users: Users) => void = () => undefined) {
  const users = usersSchema.parse(structuredClone(apiSamples.users));
  change(users);
  const client = testQueryClient();
  client.setQueryDefaults(usersKey, { staleTime: Infinity });
  client.setQueryData(usersKey, { data: users, revision: '"r1"' });
  renderWithProviders(
    <>
      <UsersScreen />
      <Toaster />
    </>,
    client,
  );
}

function row(name: string) {
  return screen.getByRole("cell", { name: new RegExp(`^${name}`) }).closest("tr") as HTMLElement;
}

afterEach(() => {
  vi.unstubAllGlobals();
});

it("adding a user sends the name and password with the loaded revision and lists them", async () => {
  const answered = usersSchema.parse(structuredClone(apiSamples.users));
  answered.users.push({ name: "bob", you: false });
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
  expect(JSON.parse(String(init.body))).toEqual({ name: "bob", password: "correct horse" });
  expect(await screen.findByText("bob can now sign in")).toBeInTheDocument();
  expect(screen.getByRole("cell", { name: "bob" })).toBeInTheDocument();
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
  expect(screen.getByRole("cell", { name: "anna" })).toBeInTheDocument();
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
