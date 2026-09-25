import { request } from "@/shared/api";
import { api } from "@/shared/config";

import { portalSchema } from "../model/schema";

export async function fetchPortal() {
  return (await request(api.publicPortal, { schema: portalSchema, redirectOnUnauthorized: false })).data;
}
