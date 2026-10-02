import { request } from "@/shared/api";
import { api } from "@/shared/config";

import { type WidgetAnswer, widgetActedSchema, widgetAnswerSchema, widgetPreviewSchema } from "../model/schema";

export async function fetchWidgetData(id: string, scope: "private" | "public"): Promise<WidgetAnswer> {
  const path = scope === "public" ? api.publicWidget(id) : api.widget(id);
  return (await request(path, { schema: widgetAnswerSchema, redirectOnUnauthorized: scope === "private" })).data;
}

export async function pressWidgetAction(id: string, action: string) {
  return (await request(api.widgetAction(id, action), { method: "POST", body: {}, schema: widgetActedSchema })).data;
}

export type PreviewAsk = { id: string | null; settings: Record<string, unknown>; run: boolean; sample: unknown };

export async function previewWidget(ask: PreviewAsk) {
  return (await request(api.widgetPreview, { method: "POST", body: ask, schema: widgetPreviewSchema })).data;
}
