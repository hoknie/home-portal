import { expect, it } from "vitest";

import { byField, formPathOf } from "./server-errors";

it("a list the form edits keeps its index, a list it does not is named as a whole", () => {
  expect(formPathOf("links[1].url")).toBe("links.1.url");
  expect(formPathOf("environments[0]")).toBe("environments");
  expect(formPathOf("probe.port")).toBe("probe.port");
});

it("puts the publication's errors on the publication fields", () => {
  expect(formPathOf("proxy.host")).toBe("publication.host");
  expect(formPathOf("proxy.environments[0]")).toBe("publication.environments");
  expect(formPathOf("proxy.auth[1]")).toBe("publication.auth");
  expect(formPathOf("proxy.tls.certificate")).toBe("publication.certificate");
  expect(formPathOf("proxy.tls.email")).toBe("publication.email");
});


it("keeps an error the form has no field for, so that it can still be shown", () => {
  const { placed, unplaced } = byField(
    [
      { field: "proxy.auth", message: "must be under proxy.cookie_domain" },
      { field: "services[2].proxy.auth", message: "must be under proxy.cookie_domain" },
    ],
    ["name", "publication"],
  );
  expect(placed).toEqual([{ path: "publication.auth", message: "must be under proxy.cookie_domain" }]);
  expect(unplaced).toEqual([{ field: "services[2].proxy.auth", message: "must be under proxy.cookie_domain" }]);
});
