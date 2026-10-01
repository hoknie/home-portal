import { describe, expect, it } from "vitest";

import { apiSamples } from "@/shared/api";

import { row } from "./address-rows";
import { emptyServiceForm as emptyOf, formOf as formWith, groupsOf, requestOf, serviceFormSchema } from "./form";
import { servicesSchema } from "./schema";

const CONFIGURED = ["local", "vpn"];
const emptyServiceForm = emptyOf(CONFIGURED);
const formOf = (service: Parameters<typeof formWith>[0]) => formWith(service, CONFIGURED);

function at(url: string, rest: object = {}) {
  return { ...emptyServiceForm, rows: [row({ environments: ["local", "vpn"], address: url })], ...rest };
}

function messages(form: unknown) {
  const result = serviceFormSchema.safeParse(form);
  return result.success ? {} : Object.fromEntries(result.error.issues.map((issue) => [issue.path.join("."), issue.message]));
}

describe("the service form mirrors the server rules", () => {
  it("accepts a minimal valid service", () => {
    expect(messages(at("http://10.0.0.5", { id: "media", name: "Media" }))).toEqual({});
  });

  it("names every invalid field with a message key", () => {
    const form = at("ftp://x", { id: "Bad Id", name: "", probe: { ...emptyServiceForm.probe, path: "health", every_seconds: 4 } });
    expect(messages(form)).toMatchObject({
      id: "validation.id",
      name: "validation.name",
      "rows.0.address": "validation.url",
      "probe.path": "validation.probePath",
      "probe.every_seconds": "validation.probeEvery",
    });
  });

  it("requires a timeout shorter than the period", () => {
    const form = at("http://a", { id: "a", name: "A", probe: { ...emptyServiceForm.probe, every_seconds: 10, timeout_seconds: 10 } });
    expect(messages(form)).toEqual({ "probe.timeout_seconds": "validation.probeTimeout" });
  });

  it("round-trips a service from the API into a request with blanks as null", () => {
    const service = servicesSchema.parse(apiSamples.services).services[1];
    expect(requestOf(formOf(service))).toMatchObject({ id: "nas", group: null, icon: null, description: null, probe: { every_seconds: 60 } });
  });
});

describe("fields the form does not edit survive an edit", () => {
  it("keeps addresses, visibility, flags and the probe kind and port of a service", () => {
    const service = servicesSchema.parse(apiSamples.services).services[2];
    const edited = {
      ...service,
      addresses: { local: "tcp://192.168.1.30" },
      environments: ["local"],
      public: true,
      public_status: true,
      notify: false,
    };
    const request = requestOf(serviceFormSchema.parse(formOf(edited)));
    expect(request).toMatchObject({
      addresses: { local: "tcp://192.168.1.30" },
      environments: ["local"],
      public: true,
      public_status: true,
      notify: false,
      probe: { kind: "tcp", port: 9100, environment: null },
    });
  });

  it("keeps links, notes and related widgets", () => {
    const service = servicesSchema.parse(apiSamples.services).services[0];
    const request = requestOf(serviceFormSchema.parse(formOf(service)));
    expect(request.links).toEqual([{ title: "Admin", url: "http://192.168.1.10:8096/web/#/dashboard" }]);
    expect(request.notes).toContain("docker restart jellyfin");
    expect(request.widgets).toEqual(["box"]);
  });

  it("sends no explicit notify for a service that keeps the default", () => {
    const service = servicesSchema.parse(apiSamples.services).services[0];
    expect(requestOf(formOf(service)).notify).toBeNull();
  });
});

describe("the address follows the probe kind", () => {
  const base = { ...emptyServiceForm, id: "printer", name: "Printer" };
  const with_ = (url: string, probe: object = {}) => at(url, { id: "printer", name: "Printer", probe: { ...base.probe, ...probe } });

  it("accepts any scheme with a host for tcp and icmp, and only http for http", () => {
    expect(messages(with_("ssh://nas.home.lan", { kind: "tcp" }))).toEqual({});
    expect(messages(with_("icmp://printer.home.lan", { kind: "icmp" }))).toEqual({});
    expect(messages(with_("ssh://nas.home.lan"))).toEqual({ "rows.0.address": "validation.url" });
  });

  it("asks for a port when a tcp address has none", () => {
    expect(messages(with_("tcp://printer.home.lan", { kind: "tcp" }))).toEqual({
      "probe.port": "validation.probePort",
    });
    expect(messages(with_("tcp://printer.home.lan", { kind: "tcp", port: 9100 }))).toEqual({});
  });
});

describe("the status carries a diagnosis", () => {
  it("reads a known code and turns an unknown one into other", () => {
    const services = servicesSchema.parse(apiSamples.services).services;
    expect(services[1].status.diagnosis).toBe("refused");
    expect(services[0].status.diagnosis).toBeNull();
    const future = { ...apiSamples.services, services: [{ ...apiSamples.services.services[1], status: { ...apiSamples.services.services[1].status, diagnosis: "cosmic-rays" } }] };
    expect(servicesSchema.parse(future).services[0].status.diagnosis).toBe("other");
  });
});

it("the known groups are the distinct non-empty groups of the services, in the order of the language", () => {
  expect(groupsOf([{ group: "Network" }, { group: "Media" }, { group: null }, { group: " " }, { group: "Network" }, { group: "Archive" }], "en")).toEqual([
    "Archive",
    "Media",
    "Network",
  ]);
});

it("sorts group names by the rules of the page's language", () => {
  const groups = [{ group: "Ñandú" }, { group: "Nube" }];
  expect(groupsOf(groups, "es")).toEqual(["Nube", "Ñandú"]);
  expect(groupsOf(groups, "en")).toEqual(["Ñandú", "Nube"]);
});
