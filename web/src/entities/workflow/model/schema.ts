import { z } from "zod";

import { runSchema } from "@/entities/automation/@x/workflow";

import type { Operation } from "./transforming/operations";

export type Condition = {
  left?: string;
  op?: string;
  right?: string;
  all?: Condition[];
  any?: Condition[];
};

export const conditionSchema: z.ZodType<Condition> = z.lazy(() =>
  z.object({
    left: z.string().optional(),
    op: z.string().optional(),
    right: z.string().optional(),
    all: z.array(conditionSchema).optional(),
    any: z.array(conditionSchema).optional(),
  }),
);

export const operationSchema: z.ZodType<Operation> = z.lazy(() =>
  z.object({
    op: z.string(),
    args: z.array(z.unknown()).optional(),
    where: conditionSchema.optional(),
    to: z.string().optional(),
    key: z.string().optional(),
    order: z.string().optional(),
    operations: z.array(operationSchema).optional(),
  }),
);

export type Step = {
  id: string;
  kind: string;
  label?: string;
  condition?: Condition;
  then?: Step[];
  else?: Step[];
  repeat?: number;
  for_each?: string;
  while?: Condition;
  max_iterations?: number;
  body?: Step[] | string;
  branches?: Step[][];
  workflow?: string;
  inputs?: Record<string, string>;
  outcome?: string;
  reason?: string;
  variable?: string;
  value?: string;
  json?: string;
  list?: string[];
  object?: Record<string, string>;
  seconds?: number;
  method?: string;
  url?: string;
  headers?: Record<string, string>;
  timeout_seconds?: number;
  fail_on_error?: boolean;
  response_sample?: string;
  script?: string;
  args?: string[];
  env?: Record<string, string>;
  stdin?: string;
  text?: string;
  title?: string;
  channel?: string;
  message?: string;
  level?: string;
  service?: string;
  input?: string;
  operations?: Operation[];
  automation?: string;
  fields?: Record<string, string>;
  wait?: boolean;
};

export const stepSchema: z.ZodType<Step> = z.lazy(() =>
  z.object({
    id: z.string(),
    kind: z.string(),
    label: z.string().optional(),
    condition: conditionSchema.optional(),
    then: z.array(stepSchema).optional(),
    else: z.array(stepSchema).optional(),
    repeat: z.number().optional(),
    for_each: z.string().optional(),
    while: conditionSchema.optional(),
    max_iterations: z.number().optional(),
    body: z.union([z.array(stepSchema), z.string()]).optional(),
    branches: z.array(z.array(stepSchema)).optional(),
    workflow: z.string().optional(),
    inputs: z.record(z.string(), z.string()).optional(),
    outcome: z.string().optional(),
    reason: z.string().optional(),
    variable: z.string().optional(),
    value: z.string().optional(),
    json: z.string().optional(),
    list: z.array(z.string()).optional(),
    object: z.record(z.string(), z.string()).optional(),
    seconds: z.number().optional(),
    method: z.string().optional(),
    url: z.string().optional(),
    headers: z.record(z.string(), z.string()).optional(),
    timeout_seconds: z.number().optional(),
    fail_on_error: z.boolean().optional(),
    response_sample: z.string().optional(),
    script: z.string().optional(),
    args: z.array(z.string()).optional(),
    env: z.record(z.string(), z.string()).optional(),
    stdin: z.string().optional(),
    text: z.string().optional(),
    title: z.string().optional(),
    channel: z.string().optional(),
    message: z.string().optional(),
    level: z.string().optional(),
    service: z.string().optional(),
    input: z.string().optional(),
    operations: z.array(operationSchema).optional(),
    automation: z.string().optional(),
    fields: z.record(z.string(), z.string()).optional(),
    wait: z.boolean().optional(),
  }),
);

export const USAGE_KINDS = ["automation", "webhook", "workflow"] as const;

export const usageSchema = z.object({ kind: z.string(), id: z.string(), title: z.string() });

export type Usage = z.infer<typeof usageSchema>;

export const INPUT_TYPES = ["text", "number", "boolean", "list", "object"] as const;

export type InputType = (typeof INPUT_TYPES)[number];

export const inputDeclarationSchema = z.preprocess(
  (value) => (typeof value === "string" ? { name: value } : value),
  z.object({
    name: z.string(),
    type: z.enum(INPUT_TYPES).default("text"),
    default: z.unknown().nullable().default(null),
    description: z.string().nullable().default(null),
  }),
);

export type InputDeclaration = z.infer<typeof inputDeclarationSchema>;

export const workflowSchema = z.object({
  id: z.string(),
  title: z.string(),
  enabled: z.boolean(),
  description: z.string().nullable().default(null),
  tags: z.array(z.string()).default([]),
  timeout_seconds: z.number(),
  inputs: z.array(inputDeclarationSchema).default([]),
  steps: z.array(stepSchema),
  steps_version: z.string().default(""),
  used_by: z.array(usageSchema).default([]),
  last_run: runSchema.nullable().default(null),
  active_run: runSchema.nullable().default(null),
});

export type Workflow = z.infer<typeof workflowSchema>;

export const workflowsSchema = z.object({ workflows: z.array(workflowSchema) });

export const FIELD_TYPES = [
  "template",
  "template-list",
  "template-table",
  "integer",
  "boolean",
  "choice",
  "condition",
  "steps",
  "branches",
  "name",
  "workflow",
  "script",
  "sample",
  "channel",
  "operations",
  "automation",
] as const;

export type FieldType = (typeof FIELD_TYPES)[number];

export const KIND_GROUPS = ["flow", "data", "actions"] as const;

export type KindGroup = (typeof KIND_GROUPS)[number];

export const kindFieldSchema = z.object({
  name: z.string(),
  type: z.enum(FIELD_TYPES),
  required: z.boolean(),
  default: z.string().nullable(),
  minimum: z.number().nullable(),
  maximum: z.number().nullable(),
  choices: z.array(z.string()),
});

export type KindField = z.infer<typeof kindFieldSchema>;

export const kindSchema = z.object({
  name: z.string(),
  group: z.enum(KIND_GROUPS),
  fields: z.array(kindFieldSchema),
  results: z.array(z.string()),
  exclusive: z.array(z.array(z.string())).default([]),
});

export type Kind = z.infer<typeof kindSchema>;

export const VALUE_TYPE_NAMES = ["text", "number", "boolean", "list", "object", "null", "any"] as const;

export const filterSchema = z.object({
  name: z.string(),
  arguments: z.array(z.object({ name: z.string(), type: z.string(), required: z.boolean(), choices: z.array(z.string()) })),
  accepts: z.array(z.enum(VALUE_TYPE_NAMES)),
  gives: z.enum(VALUE_TYPE_NAMES),
  element: z.boolean().default(false),
});

export type FilterDescription = z.infer<typeof filterSchema>;

export const workflowCatalogueSchema = z.object({
  kinds: z.array(kindSchema),
  operators: z.array(z.object({ name: z.string(), takes_right: z.boolean() })),
  events: z.array(z.object({ name: z.string(), fields: z.array(z.string()) })),
  filters: z.array(filterSchema).default([]),
  operations: z.array(filterSchema).default([]),
});

export type WorkflowCatalogue = z.infer<typeof workflowCatalogueSchema>;

export const workflowRunSchema = z.object({ run_id: z.string() });

export type WorkflowRequest = {
  id: string;
  title: string;
  enabled: boolean;
  description: string | null;
  tags: string[];
  timeout_seconds: number;
  inputs: InputDeclaration[];
  steps: Step[];
};

export function requestOf(workflow: Workflow): WorkflowRequest {
  return {
    id: workflow.id,
    title: workflow.title,
    enabled: workflow.enabled,
    description: workflow.description,
    tags: workflow.tags,
    timeout_seconds: workflow.timeout_seconds,
    inputs: workflow.inputs,
    steps: workflow.steps,
  };
}

export const secretNamesSchema = z.object({ secrets: z.array(z.object({ name: z.string(), set: z.boolean() })) });

export type SecretName = z.infer<typeof secretNamesSchema>["secrets"][number];
