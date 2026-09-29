import { expect, it } from "vitest";

import { apiSamples } from "@/shared/api";

import { scriptTextSchema, scriptTreeSchema } from "./schema";

it("the scripts samples parse with the schemas the page uses", () => {
  const tree = scriptTreeSchema.parse(apiSamples.scriptsTree);
  expect(tree.folders).toEqual(["media"]);
  expect(tree.files.find((file) => file.path === "tool")?.unreadable).toBe("binary");
  const text = scriptTextSchema.parse(apiSamples.scriptText);
  expect(text.entry.arguments.map((argument) => argument.name)).toEqual(["service", "--force"]);
});
