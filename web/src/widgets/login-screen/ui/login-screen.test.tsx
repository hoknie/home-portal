import { screen, waitFor } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { afterEach, beforeEach, expect, it, vi } from "vitest";

import { jsonResponse, renderWithProviders } from "@/shared/lib/testing";

import { LoginScreen } from "./login-screen";

const replace = vi.fn();
const leaveTo = vi.hoisted(() => vi.fn());
let search = "next=%2Fservices%2F";

vi.mock("next/navigation", () => ({
  useRouter: () => ({ replace }),
  useSearchParams: () => new URLSearchParams(search),
}));

vi.mock("@/shared/lib/navigation", async (original) => ({
  ...(await original<typeof import("@/shared/lib/navigation")>()),
  leaveTo,
}));

function signedOutUntilSignIn() {
  return vi.fn(async (_input: RequestInfo | URL, init?: RequestInit) =>
    init?.method === "POST" ? jsonResponse({ name: "admin" }) : new Response("sign in required", { status: 401 }),
  );
}

async function signIn() {
  await userEvent.type(await screen.findByLabelText("User name"), "admin");
  await userEvent.type(screen.getByLabelText("Password"), "secret");
  await userEvent.click(screen.getByRole("button", { name: "Sign in" }));
}

beforeEach(() => {
  replace.mockClear();
  leaveTo.mockClear();
});

afterEach(() => {
  vi.unstubAllGlobals();
});

it("returns to the page that was asked for after signing in", async () => {
  search = "next=%2Fservices%2F";
  vi.stubGlobal("fetch", signedOutUntilSignIn());
  renderWithProviders(<LoginScreen />);
  expect(screen.getByText("Sign in")).toBeInTheDocument();
  await signIn();
  await waitFor(() => expect(replace).toHaveBeenCalledWith("/services/"));
  expect(replace).toHaveBeenCalledTimes(1);
});

it("returns to a published service through the portal's check, never directly", async () => {
  search = "return=https%3A%2F%2Fnas.example.com%2Fphotos";
  vi.stubGlobal("fetch", signedOutUntilSignIn());
  renderWithProviders(<LoginScreen />);
  await signIn();
  await waitFor(() => expect(leaveTo).toHaveBeenCalledWith("/api/proxy/continue?to=https%3A%2F%2Fnas.example.com%2Fphotos"));
  expect(leaveTo).not.toHaveBeenCalledWith("https://nas.example.com/photos");
  expect(replace).not.toHaveBeenCalled();
});

it("continues at once without the form when a session already exists", async () => {
  search = "return=%2Fadmin%2Fservices%2F";
  vi.stubGlobal("fetch", vi.fn(async () => jsonResponse({ name: "admin" })));
  renderWithProviders(<LoginScreen />);
  await waitFor(() => expect(replace).toHaveBeenCalledWith("/admin/services/"));
  expect(screen.queryByLabelText("Password")).not.toBeInTheDocument();
});

it("offers the language switcher to a visitor without a session", async () => {
  search = "";
  vi.stubGlobal("fetch", signedOutUntilSignIn());
  renderWithProviders(<LoginScreen />);
  expect(await screen.findByLabelText("User name")).toBeInTheDocument();
  expect(screen.getByRole("button", { name: "Language: English" })).toBeInTheDocument();
});

it("is shown in the chosen language", async () => {
  search = "";
  vi.stubGlobal("fetch", signedOutUntilSignIn());
  renderWithProviders(<LoginScreen />, undefined, { locale: "es" });
  expect(await screen.findByLabelText("Nombre de usuario")).toBeInTheDocument();
  expect(screen.getByRole("button", { name: "Idioma: Español" })).toBeInTheDocument();
});
