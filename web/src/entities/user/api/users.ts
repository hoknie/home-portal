import { request } from "@/shared/api";
import { api } from "@/shared/config";

import { type Rights, groupsSchema } from "../model/groups";
import { usersSchema } from "../model/schema";

export function fetchUsers() {
  return request(api.users, { schema: usersSchema });
}

export function createUser(name: string, password: string, group: string | null, revision: string | null) {
  return request(api.users, { method: "POST", body: { name, password, group }, revision, schema: usersSchema });
}

export function changePassword(name: string, password: string, revision: string | null) {
  return request(api.userPassword(name), { method: "PUT", body: { password }, revision, schema: usersSchema });
}

export function changeUserGroup(name: string, group: string | null, revision: string | null) {
  return request(api.userGroup(name), { method: "PUT", body: { group }, revision, schema: usersSchema });
}

export function deleteUser(name: string, revision: string | null) {
  return request(api.user(name), { method: "DELETE", revision, schema: usersSchema });
}

export function fetchGroups() {
  return request(api.groups, { schema: groupsSchema });
}

export function createGroup(name: string, rights: Rights, revision: string | null) {
  return request(api.groups, { method: "POST", body: { name, rights }, revision, schema: groupsSchema });
}

export function changeGroup(current: string, name: string, rights: Rights, revision: string | null) {
  return request(api.group(current), { method: "PUT", body: { name, rights }, revision, schema: groupsSchema });
}

export function deleteGroup(name: string, revision: string | null) {
  return request(api.group(name), { method: "DELETE", revision, schema: groupsSchema });
}
