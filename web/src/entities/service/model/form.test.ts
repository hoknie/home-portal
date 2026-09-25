import { describe, expect, it } from "vitest";

import { apiSamples } from "@/shared/api";

import { emptyServiceForm, formOf, groupsOf, requestOf, serviceFormSchema } from "./form";
import { servicesSchema } from "./schema";

function messages(form: unknown) {
  const result = serviceFormSchema.safeParse(form);
  return result.success ? {} : Object.fromEntries(result.error.issues.map((issue) => [issue.path.join("."), issue.message]));
}

describe("the service form mirrors the server rules", () => {
  it("accepts a minimal valid service", () => {
    expect(messages({ ...emptyServiceForm, id: "media", name: "Media", url: "http://10.0.0.5" })).toEqual({});
  });

  it("names every invalid field with a message key", () => {
    const form = {
      ...emptyServiceForm,
      id: "Bad Id",
      name: "",
      url: "ftp://x",
      probe: { ...emptyServiceForm.probe, path: "health", every_seconds: 4 },
    };
    expect(messages(form)).toMatchObject({
      id: "validation.id",
      name: "validation.name",
      url: "validation.url",
      "probe.path": "validation.probePath",
      "probe.every_seconds": "validation.probeEvery",
    });
  });

  it("requires a timeout shorter than the period", () => {
    const form = { ...emptyServiceForm, id: "a", name: "A", url: "http://a", probe: { ...emptyServiceForm.probe, every_seconds: 10, timeout_seconds: 10 } };
    expect(messages(form)).toEqual({ "probe.timeout_seconds": "validation.probeTimeout" });
  });

  it("round-trips a service from the API into a request with blanks as null", () => {
    const service = servicesSchema.parse(apiSamples.services).services[1];
    expect(requestOf(formOf(service))).toMatchObject({ id: "nas", group: null, icon: null, description: null, probe: { every_seconds: 60 } });
  });
});

describe("fields the form does not edit survive an edit", () => {
  it("keeps addresses, environments, flags and the probe kind and port of a service", () => {
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

  it("accepts any scheme with a host for tcp and icmp, and only http for http", () => {
    expect(messages({ ...base, url: "ssh://nas.home.lan", probe: { ...base.probe, kind: "tcp" } })).toEqual({});
    expect(messages({ ...base, url: "icmp://printer.home.lan", probe: { ...base.probe, kind: "icmp" } })).toEqual({});
    expect(messages({ ...base, url: "ssh://nas.home.lan" })).toEqual({ url: "validation.url" });
  });

  it("asks for a port when a tcp address has none", () => {
    expect(messages({ ...base, url: "tcp://printer.home.lan", probe: { ...base.probe, kind: "tcp" } })).toEqual({
      "probe.port": "validation.probePort",
    });
    expect(messages({ ...base, url: "tcp://printer.home.lan", probe: { ...base.probe, kind: "tcp", port: 9100 } })).toEqual({});
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

it("the known groups are the distinct non-empty groups of the services, in Russian order", () => {
  expect(groupsOf([{ group: "Сеть" }, { group: "Media" }, { group: null }, { group: " " }, { group: "Сеть" }, { group: "Архив" }], "ru")).toEqual([
    "Архив",
    "Сеть",
    "Media",
  ]);
});

it("sorts group names by the rules of the page's language", () => {
  const groups = [{ group: "Ñandú" }, { group: "Nube" }];
  expect(groupsOf(groups, "es")).toEqual(["Nube", "Ñandú"]);
  expect(groupsOf(groups, "en")).toEqual(["Ñandú", "Nube"]);
});
