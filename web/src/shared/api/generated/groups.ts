import { z } from "zod";

export const groupResponseSchema = z.object({ "builtin": z.boolean(), "members": z.array(z.string()), "name": z.string(), "rights": z.record(z.string(), z.array(z.string())) });

export type GroupResponse = z.infer<typeof groupResponseSchema>;

export const areaResponseSchema = z.object({ "actions": z.array(z.string()), "area": z.string() });

export type AreaResponse = z.infer<typeof areaResponseSchema>;

export const groupsResponseSchema = z.object({ "groups": z.array(groupResponseSchema), "matrix": z.array(areaResponseSchema) });

export type GroupsResponse = z.infer<typeof groupsResponseSchema>;

export const schema = groupsResponseSchema;
