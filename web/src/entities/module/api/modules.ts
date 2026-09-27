import { request } from "@/shared/api";
import { api } from "@/shared/config";

import { type ModuleName, modulesSchema } from "../model/schema";

export function fetchModules() {
  return request(api.modules, { schema: modulesSchema });
}

export function switchModule(name: ModuleName, enabled: boolean, revision: string | null) {
  return request(api.module(name), { method: "PUT", body: { enabled }, revision, schema: modulesSchema });
}
