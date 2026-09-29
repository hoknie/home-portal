import { expect, it } from "vitest";

import { highlight } from "./highlight";

const kinds = (text: string) => highlight(text).map((tokens) => tokens.filter((token) => token.kind !== "plain").map((token) => [token.kind, token.text]));

it("marks the shebang, the header tags and the parts of an argument", () => {
  expect(kinds("#!/bin/sh\n# @arg --retries <number=3> Tries")).toEqual([
    [["shebang", "#!/bin/sh"]],
    [
      ["comment", "# "],
      ["tag", "@arg"],
      ["comment", " "],
      ["name", "--retries"],
      ["comment", " "],
      ["type", "<number=3>"],
      ["comment", " Tries"],
    ],
  ]);
});

it("marks keywords, strings, variables, numbers and trailing comments in shell", () => {
  expect(kinds('if [ "$1" = up ]; then exit 3; fi # done')).toEqual([
    [
      ["keyword", "if"],
      ["string", '"$1"'],
      ["keyword", "then"],
      ["keyword", "exit"],
      ["number", "3"],
      ["keyword", "fi"],
      ["comment", "# done"],
    ],
  ]);
});

it("a hash inside a word is not a comment, and slashes start one", () => {
  expect(kinds("echo a#b\n// @description Hi")).toEqual([
    [["keyword", "echo"]],
    [
      ["comment", "// "],
      ["tag", "@description"],
      ["comment", " Hi"],
    ],
  ]);
});

it("keeps every character, so the painted text lines up with the field", () => {
  const text = '#!/bin/sh\nset -eu\nname=${1:-nas} # x\n\necho \'a b\' "c\\"d"';
  expect(
    highlight(text)
      .map((tokens) => tokens.map((token) => token.text).join(""))
      .join("\n"),
  ).toBe(text);
});
