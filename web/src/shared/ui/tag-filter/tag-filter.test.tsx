import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { expect, it, vi } from "vitest";

import { TagFilter, tagsMatch } from "./tag-filter";

it("toggles a tag and marks the chosen ones as pressed", async () => {
  const change = vi.fn();
  render(<TagFilter label="Tags" tags={["media", "night"]} selected={["night"]} onChange={change} />);
  expect(screen.getByRole("button", { name: "night" })).toHaveAttribute("aria-pressed", "true");
  await userEvent.click(screen.getByRole("button", { name: "media" }));
  expect(change).toHaveBeenCalledWith(["night", "media"]);
});

it("keeps a record only when it has every chosen tag, ignoring case", () => {
  expect(tagsMatch(["Media", "night"], ["media"])).toBe(true);
  expect(tagsMatch(["media"], ["media", "night"])).toBe(false);
  expect(tagsMatch([], [])).toBe(true);
});

it("draws nothing without tags", () => {
  const { container } = render(<TagFilter label="Tags" tags={[]} selected={[]} onChange={vi.fn()} />);
  expect(container).toBeEmptyDOMElement();
});

it("lists each tag once ignoring case and forgets chosen tags that disappeared", async () => {
  const { distinctTags, stillChosen } = await import("./tag-filter");
  expect(distinctTags([["Backup", "media"], ["backup"]])).toEqual(["Backup", "media"]);
  expect(stillChosen(["backup", "gone"], ["Backup"])).toEqual(["backup"]);
});
