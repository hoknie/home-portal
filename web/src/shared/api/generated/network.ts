import { z } from "zod";

export const networkSettingsSchema = z.object({ "address": z.string(), "port": z.number(), "public_url": z.string().nullable(), "trusted_proxies": z.array(z.string()) });

export type NetworkSettings = z.infer<typeof networkSettingsSchema>;

export const effectiveAddressSchema = z.object({ "address": z.string(), "overridden": z.boolean() });

export type EffectiveAddress = z.infer<typeof effectiveAddressSchema>;

export const interfaceResponseSchema = z.object({ "addresses": z.array(z.string()), "name": z.string() });

export type InterfaceResponse = z.infer<typeof interfaceResponseSchema>;

export const networkResponseSchema = z.object({ "configured": networkSettingsSchema, "effective": effectiveAddressSchema, "interfaces": z.array(interfaceResponseSchema), "restart_required": z.boolean() });

export type NetworkResponse = z.infer<typeof networkResponseSchema>;

export const schema = networkResponseSchema;
