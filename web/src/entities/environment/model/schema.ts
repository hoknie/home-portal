import { z } from "zod";

export const environmentSchema = z.object({
  environment: z.string(),
  detected: z.string(),
  switchable: z.boolean(),
  environments: z.array(z.string()),
});

export type Environments = z.infer<typeof environmentSchema>;
