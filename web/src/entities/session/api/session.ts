import { emptySchema, request } from "@/shared/api";
import { api } from "@/shared/config";

import { type Credentials, sessionSchema } from "../model/schema";

export async function fetchSession() {
  return (await request(api.session, { schema: sessionSchema, redirectOnUnauthorized: false })).data;
}

export async function signIn(credentials: Credentials) {
  return (await request(api.session, { method: "POST", body: credentials, schema: sessionSchema, redirectOnUnauthorized: false })).data;
}

export async function signOut() {
  await request(api.session, { method: "DELETE", schema: emptySchema, redirectOnUnauthorized: false });
}
