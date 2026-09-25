import { request } from "@/shared/api";
import { api } from "@/shared/config";

import { environmentSchema } from "../model/schema";

export async function fetchEnvironment() {
  return (await request(api.environment, { schema: environmentSchema })).data;
}
