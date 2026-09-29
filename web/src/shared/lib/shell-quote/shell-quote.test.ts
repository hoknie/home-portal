import { expect, it } from "vitest";

import { quoted, quotedCommand } from "./shell-quote";

it.each([
  ["jellyfin", "'jellyfin'"],
  ["", "''"],
  ["it's", `'it'\\''s'`],
  ["two words", "'two words'"],
  ["$HOME", "'$HOME'"],
])("the argument %j is quoted as %s", (argument, shown) => {
  expect(quoted(argument)).toBe(shown);
});

it("a command keeps its script bare and quotes each argument apart", () => {
  expect(quotedCommand("restart.sh", ["--service", "jellyfin"])).toBe("restart.sh '--service' 'jellyfin'");
  expect(quotedCommand("restart.sh", [])).toBe("restart.sh");
});
