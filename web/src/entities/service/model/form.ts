import { z } from "zod";

import { type AddressRow, addressRowSchema, choicesOf, fieldsOf, rowProblems, rowsOf } from "./address-rows";
import { PROBE_KINDS, type ProbeKind, type Publication, type Service, TLS_MODES } from "./schema";

const TCP_DEFAULT_PORTS: Record<string, number> = {
  "http:": 80,
  "https:": 443,
  "ssh:": 22,
  "telnet:": 23,
  "smb:": 445,
  "ldap:": 389,
  "mqtt:": 1883,
  "rdp:": 3389,
  "vnc:": 5900,
  "ipp:": 631,
  "postgres:": 5432,
};

function parsed(value: string) {
  try {
    return new URL(value.trim());
  } catch {
    return null;
  }
}

export function addressAccepted(value: string, kind: ProbeKind) {
  const url = parsed(value);
  if (!url || url.hostname === "") {
    return false;
  }
  return kind !== "http" || url.protocol === "http:" || url.protocol === "https:";
}

const HOST_LABEL = /^[a-z0-9]([a-z0-9-]{0,61}[a-z0-9])?$/;

export const DEFAULT_TLS = "default";

export const PUBLICATION_TLS = [DEFAULT_TLS, ...TLS_MODES] as const;

export function hostAccepted(value: string) {
  return value.length <= 253 && value.split(".").every((label) => HOST_LABEL.test(label));
}

export function tcpPortKnown(value: string, port: number | null) {
  const url = parsed(value);
  return port !== null || (url !== null && (url.port !== "" || url.protocol in TCP_DEFAULT_PORTS));
}

export const serviceFormSchema = z
  .object({
    id: z.string().trim().regex(/^[a-z][a-z0-9-]{0,62}$/, "validation.id"),
    name: z.string().trim().min(1, "validation.name").max(80, "validation.name"),
    group: z.string().trim(),
    icon: z.string().trim(),
    description: z.string().trim().max(200, "validation.description"),
    kept: z.object({
      choices: z.array(z.string()),
      public: z.boolean(),
      public_status: z.boolean(),
      notify: z.boolean().nullable(),
      probe_environment: z.string().nullable(),
      widgets: z.array(z.string()),
    }),
    rows: z.array(addressRowSchema).min(1),
    links: z.array(
      z.object({
        title: z.string().trim().min(1, "validation.linkTitle").max(80, "validation.linkTitle"),
        url: z.string().trim().refine((value) => addressAccepted(value, "http"), "validation.linkUrl"),
      }),
    ).max(20, "validation.links"),
    notes: z.string().max(10_000, "validation.notes"),
    proxy: z.object({
      tls: z.enum(PUBLICATION_TLS),
      email: z.string().trim(),
      certificate: z.string().trim(),
      key: z.string().trim(),
      upstream_verify: z.boolean(),
    }),
    probe: z.object({
      enabled: z.boolean(),
      kind: z.enum(PROBE_KINDS),
      port: z.number().int().min(1, "validation.probePort").max(65535, "validation.probePort").nullable(),
      path: z.string().trim().startsWith("/", "validation.probePath"),
      every_seconds: z.number({ error: "validation.probeEvery" }).int("validation.probeEvery").min(5, "validation.probeEvery").max(3600, "validation.probeEvery"),
      timeout_seconds: z.number({ error: "validation.probeTimeout" }).int("validation.probeTimeout").min(1, "validation.probeTimeout").max(60, "validation.probeTimeout"),
      degraded_after_milliseconds: z
        .number({ error: "validation.probeDegraded" })
        .int("validation.probeDegraded")
        .min(1, "validation.probeDegraded")
        .max(60_000, "validation.probeDegraded"),
    }),
  })
  .refine((form) => form.probe.timeout_seconds < form.probe.every_seconds, {
    path: ["probe", "timeout_seconds"],
    message: "validation.probeTimeout",
  })
  .refine((form) => form.probe.kind !== "tcp" || !addressAccepted(mainOf(form), "tcp") || tcpPortKnown(mainOf(form), form.probe.port), {
    path: ["probe", "port"],
    message: "validation.probePort",
  })
  .refine((form) => !proxiedOf(form) || !addressAccepted(mainOf(form), form.probe.kind) || addressAccepted(probedOf(form), "http"), {
    path: ["rows", 0, "address"],
    message: "validation.mainNotHttp",
  })
  .refine((form) => !proxiedOf(form) || form.proxy.tls !== "files" || form.proxy.certificate !== "", {
    path: ["proxy", "certificate"],
    message: "validation.proxyFile",
  })
  .refine((form) => !proxiedOf(form) || form.proxy.tls !== "files" || form.proxy.key !== "", {
    path: ["proxy", "key"],
    message: "validation.proxyFile",
  })
  .superRefine((form, context) => {
    for (const problem of rowProblems(form.rows, form.kept.choices, (address) => addressAccepted(address, form.probe.kind))) {
      context.addIssue({ code: "custom", path: ["rows", problem.index, problem.field], message: problem.message });
    }
  });

type RowsForm = { rows: AddressRow[]; kept: { probe_environment: string | null; choices: string[] } };

function mainOf(form: RowsForm) {
  return form.rows[0]?.address ?? "";
}

function proxiedOf(form: RowsForm) {
  return form.rows.some((current) => current.proxied);
}

function probedOf(form: RowsForm) {
  const fields = fieldsOf(form.rows, form.kept.choices);
  const probed = form.kept.probe_environment;
  return probed !== null ? (fields.addresses[probed] ?? fields.url) : fields.url;
}

export const emptyProxy = {
  tls: DEFAULT_TLS,
  email: "",
  certificate: "",
  key: "",
  upstream_verify: true,
} satisfies ServiceForm["proxy"];

export function proxyFormOf(proxy: Publication | null): ServiceForm["proxy"] {
  if (proxy === null) {
    return { ...emptyProxy };
  }
  return {
    tls: proxy.tls?.mode ?? DEFAULT_TLS,
    email: proxy.tls?.email ?? "",
    certificate: proxy.tls?.certificate ?? "",
    key: proxy.tls?.key ?? "",
    upstream_verify: proxy.upstream_verify,
  };
}

function proxyRequestOf(settings: ServiceForm["proxy"], published: ReturnType<typeof fieldsOf>["proxy"]) {
  const blank = (value: string) => (value.trim() === "" ? null : value.trim());
  if (published === null || published.host === "") {
    return null;
  }
  const tls =
    settings.tls === DEFAULT_TLS
      ? null
      : {
          mode: settings.tls,
          email: blank(settings.email),
          certificate: settings.tls === "files" ? blank(settings.certificate) : null,
          key: settings.tls === "files" ? blank(settings.key) : null,
        };
  return { ...published, host: published.host.toLowerCase(), tls, upstream_verify: settings.upstream_verify };
}

export type ServiceForm = z.infer<typeof serviceFormSchema>;

export function emptyServiceForm(configured: readonly string[]): ServiceForm {
  const choices = choicesOf(configured);
  return {
    id: "",
    name: "",
    group: "",
    icon: "",
    description: "",
    kept: { choices, public: false, public_status: false, notify: null, probe_environment: null, widgets: [] },
    rows: [{ environments: choices.filter((name) => name !== "internet"), sign_in: false, address: "", proxied: false, locked: false, auth: null }],
    links: [],
    notes: "",
    proxy: { ...emptyProxy },
    probe: { enabled: true, kind: "http", port: null, path: "/", every_seconds: 30, timeout_seconds: 5, degraded_after_milliseconds: 1500 },
  };
}

export function formOf(service: Service, configured: readonly string[]): ServiceForm {
  const choices = choicesOf(configured);
  return {
    id: service.id,
    name: service.name,
    group: service.group ?? "",
    icon: service.icon ?? "",
    description: service.description ?? "",
    kept: {
      choices,
      public: service.public,
      public_status: service.public_status,
      notify: service.notify ? null : false,
      probe_environment: service.probe.environment,
      widgets: service.widgets,
    },
    rows: rowsOf(service, choices),
    links: service.links.map((link) => ({ ...link })),
    notes: service.notes ?? "",
    proxy: proxyFormOf(service.proxy),
    probe: {
      enabled: service.probe.enabled,
      kind: service.probe.kind,
      port: service.probe.port,
      path: service.probe.path,
      every_seconds: service.probe.every_seconds,
      timeout_seconds: service.probe.timeout_seconds,
      degraded_after_milliseconds: service.probe.degraded_after_milliseconds,
    },
  };
}

export function mainAddressOf(form: Pick<ServiceForm, "rows">) {
  return form.rows[0]?.address ?? "";
}

export function requestOf(form: ServiceForm) {
  const blank = (value: string) => (value.trim() === "" ? null : value.trim());
  const { kept, probe, rows, proxy, ...fields } = form;
  const { probe_environment, choices, ...rest } = kept;
  const placed = fieldsOf(rows, choices);
  return {
    ...fields,
    ...rest,
    url: placed.url,
    addresses: placed.addresses,
    environments: placed.environments,
    proxy: proxyRequestOf(proxy, placed.proxy),
    group: blank(form.group),
    icon: blank(form.icon),
    description: blank(form.description),
    notes: blank(form.notes),
    probe: { ...probe, environment: probe_environment },
  };
}

export function groupsOf(services: { group: string | null }[], locale: string) {
  const groups = new Set(services.map((service) => service.group?.trim() ?? "").filter((group) => group !== ""));
  return [...groups].sort((left, right) => left.localeCompare(right, locale));
}
