import { describe, expect, it } from "vitest";

import { apiSamples } from "@/shared/api";

import { newUserFormSchema, passwordFormSchema } from "./form";
import { deletable, usersSchema } from "./schema";

const users = usersSchema.parse(apiSamples.users);

describe("users", () => {
  it("the sample parses with the signed-in person marked", () => {
    expect(users.users.map((user) => [user.name, user.group, user.you])).toEqual([
      ["admin", "admin", true],
      ["anna", "family", false],
      ["guest", null, false],
    ]);
    expect(users.editable).toBe(true);
  });

  it("the last admin cannot be deleted", () => {
    const lone = { ...users, users: [{ name: "root", group: "admin", you: false }, users.users[1]] };
    expect(deletable(lone.users[0], lone)).toBe(false);
  });

  it("only another user can be deleted, and only while editable with more than one left", () => {
    expect(deletable(users.users[0], users)).toBe(false);
    expect(deletable(users.users[1], users)).toBe(true);
    expect(deletable(users.users[1], { ...users, editable: false })).toBe(false);
    expect(deletable(users.users[1], { ...users, users: [users.users[1]] })).toBe(false);
  });

  it("passwords that do not match are refused on the repeat field", () => {
    const result = newUserFormSchema.safeParse({ name: "anna", password: "correct horse", repeat: "correct hors" });
    expect(result.success).toBe(false);
    expect(result.error?.issues.map((issue) => [issue.path.join("."), issue.message])).toEqual([["repeat", "validation.userPasswordsDiffer"]]);
    expect(passwordFormSchema.safeParse({ password: "correct horse", repeat: "correct horse" }).success).toBe(true);
  });

  it("a name is trimmed and a password needs 8 to 1024 characters", () => {
    expect(newUserFormSchema.parse({ name: "  anna ", password: "12345678", repeat: "12345678" }).name).toBe("anna");
    expect(newUserFormSchema.safeParse({ name: "", password: "12345678", repeat: "12345678" }).success).toBe(false);
    expect(passwordFormSchema.safeParse({ password: "1234567", repeat: "1234567" }).success).toBe(false);
    expect(passwordFormSchema.safeParse({ password: "p".repeat(1025), repeat: "p".repeat(1025) }).success).toBe(false);
  });
});
