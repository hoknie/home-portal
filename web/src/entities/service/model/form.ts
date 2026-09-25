import { z } from "zod";

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
    url: z.string().trim(),
    group: z.string().trim(),
    icon: z.string().trim(),
    description: z.string().trim().max(200, "validation.description"),
    kept: z.object({
      addresses: z.record(z.string(), z.string()),
      environments: z.array(z.string()).nullable(),
      public: z.boolean(),
      public_status: z.boolean(),
      notify: z.boolean().nullable(),
      probe_environment: z.string().nullable(),
      widgets: z.array(z.string()),
    }),
    links: z.array(
      z.object({
        title: z.string().trim().min(1, "validation.linkTitle").max(80, "validation.linkTitle"),
        url: z.string().trim().refine((value) => addressAccepted(value, "http"), "validation.linkUrl"),
      }),
    ).max(20, "validation.links"),
    notes: z.string().max(10_000, "validation.notes"),
    publication: z.object({
      host: z.string().trim().toLowerCase().refine((value) => value === "" || hostAccepted(value), "validation.proxyHost"),
      upstream: z.string().trim().refine((value) => value === "" || addressAccepted(value, "http"), "validation.proxyUpstream"),
      environments: z.array(z.string()),
      auth: z.array(z.string()),
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
  .refine((form) => addressAccepted(form.url, form.probe.kind), {
    path: ["url"],
    message: "validation.url",
  })
  .refine((form) => form.probe.kind !== "tcp" || !addressAccepted(form.url, "tcp") || tcpPortKnown(form.url, form.probe.port), {
    path: ["probe", "port"],
    message: "validation.probePort",
  })
  .refine((form) => !published(form) || form.publication.upstream !== "" || upstreamCandidates(form).every((address) => addressAccepted(address, "http")), {
    path: ["publication", "upstream"],
    message: "validation.proxyUpstreamRequired",
  })
  .refine((form) => !published(form) || form.publication.environments.every((name) => form.kept.environments === null || form.kept.environments.includes(name)), {
    path: ["publication", "environments"],
    message: "validation.proxyEnvironments",
  })
  .refine((form) => !published(form) || form.publication.environments.every((name) => !(name in form.kept.addresses)), {
    path: ["publication", "environments"],
    message: "validation.proxyAddressGiven",
  })
  .refine((form) => !published(form) || form.publication.tls !== "files" || form.publication.certificate !== "", {
    path: ["publication", "certificate"],
    message: "validation.proxyFile",
  })
  .refine((form) => !published(form) || form.publication.tls !== "files" || form.publication.key !== "", {
    path: ["publication", "key"],
    message: "validation.proxyFile",
  });

type PublishedForm = { publication: { host: string }; url: string; kept: { addresses: Record<string, string>; probe_environment: string | null } };

function published(form: PublishedForm) {
  return form.publication.host.trim() !== "";
}

function upstreamCandidates(form: PublishedForm) {
  const probed = form.kept.probe_environment;
  if (probed !== null) {
    return [form.kept.addresses[probed] ?? form.url];
  }
  return [form.url, ...Object.values(form.kept.addresses)];
}

export const emptyPublication = {
  host: "",
  upstream: "",
  environments: ["internet"],
  auth: [],
  tls: DEFAULT_TLS,
  email: "",
  certificate: "",
  key: "",
  upstream_verify: true,
} satisfies ServiceForm["publication"];

export function publicationFormOf(proxy: Publication | null): ServiceForm["publication"] {
  if (proxy === null) {
    return { ...emptyPublication, environments: [...emptyPublication.environments] };
  }
  return {
    host: proxy.host,
    upstream: proxy.upstream ?? "",
    environments: [...proxy.environments],
    auth: [...proxy.auth],
    tls: proxy.tls?.mode ?? DEFAULT_TLS,
    email: proxy.tls?.email ?? "",
    certificate: proxy.tls?.certificate ?? "",
    key: proxy.tls?.key ?? "",
    upstream_verify: proxy.upstream_verify,
  };
}

export function publicationRequestOf(publication: ServiceForm["publication"]) {
  const blank = (value: string) => (value.trim() === "" ? null : value.trim());
  if (publication.host.trim() === "") {
    return null;
  }
  const tls =
    publication.tls === DEFAULT_TLS
      ? null
      : {
          mode: publication.tls,
          email: blank(publication.email),
          certificate: publication.tls === "files" ? blank(publication.certificate) : null,
          key: publication.tls === "files" ? blank(publication.key) : null,
        };
  return {
    host: publication.host.trim().toLowerCase(),
    upstream: blank(publication.upstream),
    environments: publication.environments,
    auth: publication.auth,
    tls,
    upstream_verify: publication.upstream_verify,
  };
}

export type ServiceForm = z.infer<typeof serviceFormSchema>;

export const emptyServiceForm: ServiceForm = {
  id: "",
  name: "",
  url: "",
  group: "",
  icon: "",
  description: "",
  kept: { addresses: {}, environments: null, public: false, public_status: false, notify: null, probe_environment: null, widgets: [] },
  links: [],
  notes: "",
  publication: emptyPublication,
  probe: { enabled: true, kind: "http", port: null, path: "/", every_seconds: 30, timeout_seconds: 5, degraded_after_milliseconds: 1500 },
};

export function formOf(service: Service): ServiceForm {
  return {
    id: service.id,
    name: service.name,
    url: service.url,
    group: service.group ?? "",
    icon: service.icon ?? "",
    description: service.description ?? "",
    kept: {
      addresses: service.addresses,
      environments: service.environments,
      public: service.public,
      public_status: service.public_status,
      notify: service.notify ? null : false,
      probe_environment: service.probe.environment,
      widgets: service.widgets,
    },
    links: service.links.map((link) => ({ ...link })),
    notes: service.notes ?? "",
    publication: publicationFormOf(service.proxy),
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

export function requestOf(form: ServiceForm) {
  const blank = (value: string) => (value.trim() === "" ? null : value.trim());
  const { kept, probe, publication, ...fields } = form;
  const { probe_environment, ...rest } = kept;
  return {
    ...fields,
    ...rest,
    proxy: publicationRequestOf(publication),
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
