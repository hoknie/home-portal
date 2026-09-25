import { request } from "@/shared/api";
import { api } from "@/shared/config";

import { type NetworkForm, networkRequestOf } from "../model/form";
import { networkSchema } from "../model/schema";

export function fetchNetwork() {
  return request(api.network, { schema: networkSchema });
}

export function saveNetwork(form: NetworkForm, revision: string | null) {
  return request(api.network, { method: "PUT", body: networkRequestOf(form), revision, schema: networkSchema });
}
