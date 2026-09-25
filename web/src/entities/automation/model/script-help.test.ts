import { expect, it } from "vitest";

import { fixOf, problemOf } from "./script-help";

it("each fixable problem gets the command that fixes it", () => {
  expect(fixOf("writable", "/srv/scripts/open.sh", 501)).toBe("chmod go-w '/srv/scripts/open.sh'");
  expect(fixOf("folder-writable", "/srv/scripts", 501)).toBe("chmod go-w '/srv/scripts'");
  expect(fixOf("not-executable", "/srv/scripts/it's.sh", 501)).toBe("chmod +x '/srv/scripts/it'\\''s.sh'");
  expect(fixOf("folder-owner", "/srv/scripts/media", 501)).toBe("sudo chown 501 '/srv/scripts/media'");
  expect(fixOf("too-deep", "/srv/scripts/a/b/c.sh", 501)).toBeNull();
});

it("an unknown code is no problem the interface can explain", () => {
  expect(problemOf("writable")).toBe("writable");
  expect(problemOf("melted")).toBeNull();
  expect(problemOf(null)).toBeNull();
});
