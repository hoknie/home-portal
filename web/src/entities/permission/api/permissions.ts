import { emptySchema, request } from "@/shared/api";
import { api } from "@/shared/config";

import { permissionsSchema } from "../model/schema";

export function fetchPermissions() {
  return request(api.permissions, { schema: permissionsSchema });
}

export function requestPermissions() {
  return request(api.permissionsRequest, { method: "POST", schema: emptySchema });
}
