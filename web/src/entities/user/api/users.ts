import { request } from "@/shared/api";
import { api } from "@/shared/config";

import { usersSchema } from "../model/schema";

export function fetchUsers() {
  return request(api.users, { schema: usersSchema });
}

export function createUser(name: string, password: string, revision: string | null) {
  return request(api.users, { method: "POST", body: { name, password }, revision, schema: usersSchema });
}

export function changePassword(name: string, password: string, revision: string | null) {
  return request(api.userPassword(name), { method: "PUT", body: { password }, revision, schema: usersSchema });
}

export function deleteUser(name: string, revision: string | null) {
  return request(api.user(name), { method: "DELETE", revision, schema: usersSchema });
}
