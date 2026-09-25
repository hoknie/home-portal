import { request } from "@/shared/api";
import { api } from "@/shared/config";

import { dashboardSchema } from "../model/schema";

export async function fetchDashboard() {
  return (await request(api.dashboard, { schema: dashboardSchema })).data;
}
