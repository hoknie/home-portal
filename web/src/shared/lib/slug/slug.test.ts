import { expect, it } from "vitest";

import { ID_LIMIT, slugOf, uniqueId } from "./slug";

it.each([
  ["Домашний NAS", "domashniy-nas"],
  ["Щука & Ёж", "shchuka-ezh"],
  ["123 Plex", "plex"],
  ["Home Assistant", "home-assistant"],
  ["  Роутер (Keenetic)  ", "router-keenetic"],
  ["!!!", ""],
])("the id proposed for %j is %j", (name, id) => {
  expect(slugOf(name)).toBe(id);
});

it("a long name is cut to the id limit without a trailing hyphen", () => {
  const id = slugOf(`${"a".repeat(62)} b ${"c".repeat(10)}`);
  expect(id.length).toBeLessThanOrEqual(ID_LIMIT);
  expect(id.endsWith("-")).toBe(false);
  expect(id).toMatch(/^[a-z][a-z0-9-]*$/);
});

it("a taken id gets the next free number", () => {
  expect(uniqueId("media", [])).toBe("media");
  expect(uniqueId("media", ["media"])).toBe("media-2");
  expect(uniqueId("media", ["media", "media-2"])).toBe("media-3");
  expect(uniqueId("", ["media"])).toBe("");
});

it("a numbered id still fits the limit", () => {
  const base = "a".repeat(ID_LIMIT);
  const id = uniqueId(base, [base]);
  expect(id.length).toBe(ID_LIMIT);
  expect(id.endsWith("-2")).toBe(true);
});
