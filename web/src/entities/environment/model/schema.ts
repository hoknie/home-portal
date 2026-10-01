import type { z } from "zod";

import { generated } from "@/shared/api";

export const environmentSchema = generated.environment.environmentResponseSchema;

export type Environments = z.infer<typeof environmentSchema>;
