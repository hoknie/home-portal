import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";

import type { Revisioned } from "@/shared/api";

import { changePassword, createUser, deleteUser, fetchUsers } from "../api/users";
import type { Users } from "./schema";

export const usersKey = ["users"] as const;

export function useUsers() {
  return useQuery({ queryKey: usersKey, queryFn: fetchUsers });
}

function useAnswering<T>(run: (input: T) => Promise<Revisioned<Users>>) {
  const client = useQueryClient();
  return useMutation({
    mutationFn: run,
    onSuccess: (answer) => client.setQueryData(usersKey, answer),
    onSettled: () => client.invalidateQueries({ queryKey: usersKey }),
  });
}

export function useCreateUser() {
  return useAnswering(({ name, password, revision }: { name: string; password: string; revision: string | null }) =>
    createUser(name, password, revision),
  );
}

export function useChangePassword() {
  return useAnswering(({ name, password, revision }: { name: string; password: string; revision: string | null }) =>
    changePassword(name, password, revision),
  );
}

export function useDeleteUser() {
  return useAnswering(({ name, revision }: { name: string; revision: string | null }) => deleteUser(name, revision));
}
