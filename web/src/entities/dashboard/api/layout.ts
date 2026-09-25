import { request } from "@/shared/api";
import { api } from "@/shared/config";

import { dashboardSchema } from "../model/schema";
import type { LayoutRequest } from "../model/layout";

export function fetchLayout() {
  return request(api.layout, { schema: dashboardSchema });
}

export function saveLayout(layout: LayoutRequest, revision: string | null) {
  return request(api.dashboard, { method: "PUT", body: layout, revision, schema: dashboardSchema });
}
