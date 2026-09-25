import { emptySchema, request, requestImage } from "@/shared/api";
import { api } from "@/shared/config";

import { requestOf, type ServiceForm } from "../model/form";
import { type HistoryRange, historySchema } from "../model/history";
import { serviceSchema, servicesSchema } from "../model/schema";

export function fetchServices() {
  return request(api.services, { schema: servicesSchema });
}

export function createService(form: ServiceForm, revision: string | null) {
  return request(api.services, { method: "POST", body: requestOf(form), revision, schema: serviceSchema });
}

export function updateService(id: string, form: ServiceForm, revision: string | null) {
  return request(api.service(id), { method: "PUT", body: requestOf(form), revision, schema: serviceSchema });
}

export function deleteService(id: string, revision: string | null) {
  return request(api.service(id), { method: "DELETE", revision, schema: servicesSchema });
}

export async function fetchHistory(id: string, range: HistoryRange) {
  return (await request(api.serviceHistory(id, range), { schema: historySchema })).data;
}

export async function probeService(id: string) {
  await request(api.serviceProbe(id), { method: "POST", body: {}, schema: emptySchema });
}

export function previewIcon(icon: string, url: string | null, signal?: AbortSignal) {
  return requestImage(api.iconPreview, { icon, url }, signal);
}
