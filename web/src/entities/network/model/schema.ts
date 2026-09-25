import { z } from "zod";

export const networkSettingsSchema = z.object({
  address: z.string(),
  port: z.number(),
  public_url: z.string().nullable(),
  trusted_proxies: z.array(z.string()),
});

export type NetworkSettings = z.infer<typeof networkSettingsSchema>;

export const networkSchema = z.object({
  configured: networkSettingsSchema,
  effective: z.object({ address: z.string(), overridden: z.boolean() }),
  restart_required: z.boolean(),
  interfaces: z.array(z.object({ name: z.string(), addresses: z.array(z.string()) })),
});

export type Network = z.infer<typeof networkSchema>;
