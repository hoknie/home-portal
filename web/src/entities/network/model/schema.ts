import type { z } from "zod";

import { generated } from "@/shared/api";

export const networkSettingsSchema = generated.network.networkSettingsSchema;

export type NetworkSettings = z.infer<typeof networkSettingsSchema>;

export const networkSchema = generated.network.networkResponseSchema;

export type Network = z.infer<typeof networkSchema>;
