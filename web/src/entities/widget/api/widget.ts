import { request } from "@/shared/api";
import { api } from "@/shared/config";

import { widgetDataSchema } from "../model/schema";

export async function fetchWidgetData(id: string, scope: "private" | "public") {
  const path = scope === "public" ? api.publicWidget(id) : api.widget(id);
  return (await request(path, { schema: widgetDataSchema, redirectOnUnauthorized: scope === "private" })).data;
}
