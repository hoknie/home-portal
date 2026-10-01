import { expect, it } from "vitest";

import { row } from "@/entities/service";

import { byField, formPathOf } from "./server-errors";

const ROWS = [
  row({ environments: ["local"], address: "http://192.168.1.50" }),
  row({ environments: ["vpn"], address: "http://10.8.0.5" }),
  row({ environments: ["internet"], address: "https://nas.example.com", proxied: true }),
];

it("a list the form edits keeps its index, a list it does not is named as a whole", () => {
  expect(formPathOf("links[1].url", ROWS)).toBe("links.1.url");
  expect(formPathOf("probe.port", ROWS)).toBe("probe.port");
  expect(formPathOf("widgets[0]", ROWS)).toBe("widgets");
});

it("puts address errors on the row and field they name", () => {
  expect(formPathOf("url", ROWS)).toBe("rows.0.address");
  expect(formPathOf("environments[0]", ROWS)).toBe("rows.0.environments");
  expect(formPathOf("addresses.vpn", ROWS)).toBe("rows.1.address");
  expect(formPathOf("proxy.host", ROWS)).toBe("rows.2.address");
  expect(formPathOf("proxy.environments[0]", ROWS)).toBe("rows.2.environments");
  expect(formPathOf("proxy.auth[1]", ROWS)).toBe("rows.2.sign_in");
});

it("puts the certificate's errors on the proxy settings", () => {
  expect(formPathOf("proxy.tls.certificate", ROWS)).toBe("proxy.certificate");
  expect(formPathOf("proxy.tls.email", ROWS)).toBe("proxy.email");
});

it("keeps an error the form has no field for, so that it can still be shown", () => {
  const { placed, unplaced } = byField(
    [
      { field: "proxy.auth", message: "must be under proxy.cookie_domain" },
      { field: "services[2].proxy.auth", message: "must be under proxy.cookie_domain" },
    ],
    ["name", "rows"],
    ROWS,
  );
  expect(placed).toEqual([{ path: "rows.2.sign_in", message: "must be under proxy.cookie_domain" }]);
  expect(unplaced).toEqual([{ field: "services[2].proxy.auth", message: "must be under proxy.cookie_domain" }]);
});
