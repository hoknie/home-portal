import { screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { afterEach, expect, it, vi } from "vitest";

import { jsonResponse, renderWithProviders } from "@/shared/lib/testing";

import { SignInForm } from "./sign-in-form";

afterEach(() => {
  vi.unstubAllGlobals();
});

async function submitWith(response: Response) {
  vi.stubGlobal("fetch", vi.fn(async () => response));
  const onSignedIn = vi.fn();
  renderWithProviders(<SignInForm onSignedIn={onSignedIn} />);
  await userEvent.type(screen.getByLabelText("User name"), "admin");
  await userEvent.type(screen.getByLabelText("Password"), "secret");
  await userEvent.click(screen.getByRole("button", { name: "Sign in" }));
  return onSignedIn;
}

it("signs in and reports it", async () => {
  const onSignedIn = await submitWith(jsonResponse({ name: "admin" }));
  expect(onSignedIn).toHaveBeenCalledOnce();
});

it("says the name or password is wrong on 401", async () => {
  const onSignedIn = await submitWith(new Response("sign in required", { status: 401 }));
  expect(await screen.findByText("Wrong user name or password")).toBeInTheDocument();
  expect(onSignedIn).not.toHaveBeenCalled();
});

it("says how long to wait on 429", async () => {
  await submitWith(new Response("slow down", { status: 429, headers: { "Retry-After": "42" } }));
  expect(await screen.findByText("Too many attempts. Try again in 42 s.")).toBeInTheDocument();
});

it("asks for both fields before sending anything", async () => {
  const fetch = vi.fn();
  vi.stubGlobal("fetch", fetch);
  renderWithProviders(<SignInForm onSignedIn={vi.fn()} />);
  await userEvent.click(screen.getByRole("button", { name: "Sign in" }));
  expect(await screen.findAllByText("Required")).toHaveLength(2);
  expect(fetch).not.toHaveBeenCalled();
});
