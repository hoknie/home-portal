import { z } from "zod";

import { request } from "@/shared/api";
import { api } from "@/shared/config";

import { scriptEntrySchema, scriptTextSchema, scriptTreeSchema } from "../model/schema";

const nothing = z.null();

export async function fetchScripts() {
  return (await request(api.scripts, { schema: scriptTreeSchema })).data;
}

export async function fetchScript(path: string) {
  return (await request(api.scriptFile(path), { schema: scriptTextSchema })).data;
}

export function saveScript(path: string, content: string, revision: string) {
  return request(api.scriptFile(path), {
    method: "PUT",
    body: { content },
    revision: `"${revision}"`,
    schema: scriptEntrySchema,
  });
}

export function createScript(path: string, content: string) {
  return request(api.scriptFile(), {
    method: "POST",
    body: { path, content },
    schema: scriptEntrySchema,
  });
}

export async function deleteScript(path: string, revision: string) {
  await request(api.scriptFile(path), {
    method: "DELETE",
    revision: `"${revision}"`,
    schema: nothing,
  });
}

export function moveScript(from: string, to: string, revision: string) {
  return request(api.scriptMove, {
    method: "POST",
    body: { from, to },
    revision: `"${revision}"`,
    schema: scriptEntrySchema,
  });
}

export async function createFolder(name: string | null) {
  await request(api.scriptFolder(), {
    method: "POST",
    body: name === null ? {} : { name },
    schema: nothing,
  });
}

export async function deleteFolder(name: string) {
  await request(api.scriptFolder(name), { method: "DELETE", schema: nothing });
}
