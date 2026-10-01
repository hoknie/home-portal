import { request } from "@/shared/api";
import { api } from "@/shared/config";

import { type PreviewQuestion, transformPreviewSchema } from "../model/transforming/previews";

export async function previewTransform(question: PreviewQuestion) {
  return (await request(api.transformPreview, { method: "POST", body: question, schema: transformPreviewSchema })).data;
}
