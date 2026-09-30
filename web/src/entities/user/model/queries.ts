import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";

import type { Revisioned } from "@/shared/api";

import { changeGroup, changePassword, changeUserGroup, createGroup, createUser, deleteGroup, deleteUser, fetchGroups, fetchUsers } from "../api/users";
import type { Groups, Rights } from "./groups";
import type { Users } from "./schema";

export const usersKey = ["users"] as const;

export const groupsKey = ["groups"] as const;

const SESSION_KEY = ["session"] as const;

export function useUsers() {
  return useQuery({ queryKey: usersKey, queryFn: fetchUsers });
}

export function useGroups(enabled = true) {
  return useQuery({ queryKey: groupsKey, queryFn: fetchGroups, enabled });
}

function useAnswering<T>(run: (input: T) => Promise<Revisioned<Users>>) {
  const client = useQueryClient();
  return useMutation({
    mutationFn: run,
    onSuccess: (answer) => client.setQueryData(usersKey, answer),
    onSettled: () => {
      void client.invalidateQueries({ queryKey: usersKey });
      void client.invalidateQueries({ queryKey: groupsKey });
      void client.invalidateQueries({ queryKey: SESSION_KEY });
    },
  });
}

function useGroupWrite<T>(run: (input: T) => Promise<Revisioned<Groups>>) {
  const client = useQueryClient();
  return useMutation({
    mutationFn: run,
    onSuccess: (answer) => client.setQueryData(groupsKey, answer),
    onSettled: () => {
      void client.invalidateQueries({ queryKey: groupsKey });
      void client.invalidateQueries({ queryKey: usersKey });
      void client.invalidateQueries({ queryKey: SESSION_KEY });
    },
  });
}

export function useCreateUser() {
  return useAnswering(({ name, password, group, revision }: { name: string; password: string; group: string | null; revision: string | null }) =>
    createUser(name, password, group, revision),
  );
}

export function useChangePassword() {
  return useAnswering(({ name, password, current, revision }: { name: string; password: string; current: string | null; revision: string | null }) =>
    changePassword(name, password, current, revision),
  );
}

export function useChangeUserGroup() {
  return useAnswering(({ name, group, revision }: { name: string; group: string | null; revision: string | null }) =>
    changeUserGroup(name, group, revision),
  );
}

export function useDeleteUser() {
  return useAnswering(({ name, revision }: { name: string; revision: string | null }) => deleteUser(name, revision));
}

export function useCreateGroup() {
  return useGroupWrite(({ name, rights, revision }: { name: string; rights: Rights; revision: string | null }) => createGroup(name, rights, revision));
}

export function useChangeGroup() {
  return useGroupWrite(({ current, name, rights, revision }: { current: string; name: string; rights: Rights; revision: string | null }) =>
    changeGroup(current, name, rights, revision),
  );
}

export function useDeleteGroup() {
  return useGroupWrite(({ name, revision }: { name: string; revision: string | null }) => deleteGroup(name, revision));
}
