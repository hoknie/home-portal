import { emptySchema, request } from "@/shared/api";
import { api } from "@/shared/config";

export async function restartPortal() {
  await request(api.restartPortal, { method: "POST", body: {}, schema: emptySchema });
}
