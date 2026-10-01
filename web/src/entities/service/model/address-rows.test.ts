import { describe, expect, it } from "vitest";

import { apiSamples } from "@/shared/api";

import { type AddressRow, choicesOf, fieldsOf, row, rowProblems, rowsOf } from "./address-rows";
import { servicesSchema } from "./schema";

const ALL = choicesOf(["local", "vpn"]);

function placed(service: Parameters<typeof rowsOf>[0]) {
  return { url: service.url, addresses: service.addresses, environments: service.environments, proxy: service.proxy ? { host: service.proxy.host, environments: service.proxy.environments, auth: service.proxy.auth } : null };
}

function problems(rows: AddressRow[]) {
  return rowProblems(rows, ALL, (address) => address.startsWith("http")).map((problem) => `${problem.index}.${problem.field}: ${problem.message}`);
}

describe("address rows", () => {
  it("every sample service comes back unchanged through the rows", () => {
    for (const service of servicesSchema.parse(apiSamples.services).services) {
      expect(fieldsOf(rowsOf(service, ALL), ALL), service.id).toEqual(placed(service));
    }
  });

  it("a main address, a direct vpn address and a published host become three rows", () => {
    const service = { url: "http://192.168.1.50", addresses: { vpn: "http://10.8.0.5" }, environments: null, proxy: { host: "nas.example.com", environments: ["internet"], auth: [], tls: null, upstream_verify: true } };
    const rows = rowsOf(service, ALL);
    expect(rows.map((current) => [current.environments, current.address, current.proxied])).toEqual([
      [["local"], "http://192.168.1.50", false],
      [["vpn"], "http://10.8.0.5", false],
      [["internet"], "https://nas.example.com", true],
    ]);
    expect(fieldsOf(rows, ALL)).toEqual(placed(service));
  });

  it("environments in no row hide the service there", () => {
    const rows = [row({ environments: ["local"], address: "http://192.168.1.50" }), row({ environments: ["vpn"], address: "http://10.8.0.5" })];
    expect(fieldsOf(rows, ALL).environments).toEqual(["local", "vpn"]);
  });

  it("sign-in from some of the published environments only stays as a read-only row and is written back", () => {
    const service = { url: "http://10.0.0.5", addresses: {}, environments: null, proxy: { host: "media.example.com", environments: ["vpn", "internet"], auth: ["internet"], tls: null, upstream_verify: true } };
    const rows = rowsOf(service, ALL);
    expect(rows[1]).toMatchObject({ proxied: true, locked: true, auth: ["internet"] });
    expect(fieldsOf(rows, ALL)).toEqual(placed(service));
  });

  it("an address for a hidden environment stays as a read-only row and is written back", () => {
    const service = { url: "http://10.0.0.5", addresses: { internet: "https://old.example.com" }, environments: ["local"], proxy: null };
    const rows = rowsOf(service, ALL);
    expect(rows.at(-1)).toMatchObject({ locked: true, environments: ["internet"] });
    expect(fieldsOf(rows, ALL)).toEqual(placed(service));
  });

  it("each rule of the table names its row and field", () => {
    const main = row({ environments: ["local"], address: "http://192.168.1.50" });
    expect(problems([main, row({ environments: ["vpn"], address: "http://10.8.0.5", sign_in: true })])).toEqual(["1.sign_in: validation.signInNeedsProxy"]);
    expect(problems([main, row({ environments: [], address: "http://10.8.0.5" })])).toEqual(["1.environments: validation.rowEnvironments"]);
    expect(problems([main, row({ environments: ["local"], address: "http://10.8.0.5" })])).toEqual(["1.environments: validation.rowEnvironmentTaken"]);
    expect(problems([main, row({ environments: ["vpn"], address: "https://a.example.com", proxied: true }), row({ environments: ["internet"], address: "https://b.example.com", proxied: true })])).toEqual([
      "2.proxied: validation.oneProxiedRow",
    ]);
    expect(problems([main, row({ environments: ["internet"], address: "https://a.example.com/path", proxied: true })])).toEqual(["1.address: validation.proxyHost"]);
    expect(problems([row({ ...main, proxied: true })])).toEqual(["0.proxied: validation.mainNotProxied"]);
    const four = [main, row({ environments: ["vpn"], address: "http://a" }), row({ environments: ["internet"], address: "http://b" }), row({ environments: [], address: "http://c" })];
    expect(problems(four)).toContain("3.environments: validation.tooManyRows");
  });
});
