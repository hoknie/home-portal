import { screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { afterEach, expect, it, vi } from "vitest";

import { renderWithProviders } from "@/shared/lib/testing";

import { LANGUAGE_COOKIE, languageCookie, page } from "../model/choose";
import { LanguageSwitch } from "./language-switch";

const reload = vi.spyOn(page, "reload").mockImplementation(() => undefined);

async function open(name: string) {
  screen.getByRole("button", { name }).focus();
  await userEvent.keyboard("{Enter}");
}

afterEach(() => {
  document.cookie = `${LANGUAGE_COOKIE}=; Path=/; Max-Age=0`;
  reload.mockClear();
});

it("choosing Spanish remembers it for a year and reloads the page", async () => {
  renderWithProviders(<LanguageSwitch />);
  await open("Language: English");
  await userEvent.click(screen.getByRole("menuitemradio", { name: "Español" }));
  expect(document.cookie).toContain("portal_language=es");
  expect(reload).toHaveBeenCalledOnce();
});

it("the cookie is shared by the whole portal, kept a year and sent on navigation", () => {
  expect(languageCookie("es")).toBe("portal_language=es; Path=/; Max-Age=31536000; SameSite=Lax");
});

it("choosing the current language does nothing", async () => {
  renderWithProviders(<LanguageSwitch />);
  await open("Language: English");
  await userEvent.click(screen.getByRole("menuitemradio", { name: "English" }));
  expect(reload).not.toHaveBeenCalled();
});

it("lists every language by its own name and marks the current one", async () => {
  renderWithProviders(<LanguageSwitch />, undefined, { locale: "en" });
  await open("Language: English");
  expect(screen.getByRole("menuitemradio", { name: "English" })).toHaveAttribute("aria-checked", "true");
  expect(screen.getByRole("menuitemradio", { name: "Русский" })).toHaveAttribute("aria-checked", "false");
  expect(screen.getByRole("menuitemradio", { name: "Español" })).toHaveAttribute("lang", "es");
  await userEvent.keyboard("{Escape}");
});
