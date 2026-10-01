import { z } from "zod";

import { runSchema } from "@/entities/automation/@x/workflow";
import { generated } from "@/shared/api";

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
  repeat?: number | string;
  for_each?: string;
  while?: Condition;
  max_iterations?: number | string;
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
  seconds?: number | string;
  method?: string;
  url?: string;
  headers?: Record<string, string>;
  timeout_seconds?: number | string;
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
    repeat: z.union([z.number(), z.string()]).optional(),
    for_each: z.string().optional(),
    while: conditionSchema.optional(),
    max_iterations: z.union([z.number(), z.string()]).optional(),
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
    seconds: z.union([z.number(), z.string()]).optional(),
    method: z.string().optional(),
    url: z.string().optional(),
    headers: z.record(z.string(), z.string()).optional(),
    timeout_seconds: z.union([z.number(), z.string()]).optional(),
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

export const usageSchema = generated.workflows.workflowUsageResponseSchema;

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

export const workflowSchema = generated.workflows.workflowResponseSchema.extend({
  inputs: z.array(inputDeclarationSchema),
  steps: z.array(stepSchema),
  last_run: runSchema.nullable(),
  active_run: runSchema.nullable(),
});

export type Workflow = z.infer<typeof workflowSchema>;

export const workflowsSchema = generated.workflows.workflowsResponseSchema.extend({ workflows: z.array(workflowSchema) });

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

const catalogue = generated.workflowCatalogue;

export const kindFieldSchema = catalogue.stepFieldResponseSchema.extend({ type: z.enum(FIELD_TYPES) });

export type KindField = z.infer<typeof kindFieldSchema>;

export const kindSchema = catalogue.stepKindResponseSchema.extend({ group: z.enum(KIND_GROUPS), fields: z.array(kindFieldSchema) });

export type Kind = z.infer<typeof kindSchema>;

export const VALUE_TYPE_NAMES = ["text", "number", "boolean", "list", "object", "null", "any"] as const;

export const filterSchema = catalogue.filterResponseSchema.extend({ accepts: z.array(z.enum(VALUE_TYPE_NAMES)), gives: z.enum(VALUE_TYPE_NAMES) });

export type FilterDescription = z.infer<typeof filterSchema>;

export const workflowCatalogueSchema = catalogue.workflowCatalogueResponseSchema.extend({
  kinds: z.array(kindSchema),
  filters: z.array(filterSchema),
  operations: z.array(filterSchema),
});

export type WorkflowCatalogue = z.infer<typeof workflowCatalogueSchema>;

export const workflowRunSchema = generated.workflowRun.queuedResponseSchema;

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

export const secretNamesSchema = generated.secrets.secretsResponseSchema;

export type SecretName = z.infer<typeof secretNamesSchema>["secrets"][number];
