import { z } from "zod";

import * as services from "./generated/services";

export const fieldErrorSchema = z.object({ field: z.string(), message: z.string() });

export const fieldErrorsSchema = z.object({ errors: z.array(fieldErrorSchema) });

export const emptySchema = z.unknown();

export const serviceStateSchema = services.serviceStateSchema;

export type ServiceState = z.infer<typeof serviceStateSchema>;

export const DIAGNOSES = services.diagnosisSchema.unwrap().options;

export const diagnosisSchema = services.diagnosisSchema;

export type Diagnosis = z.infer<typeof diagnosisSchema>;

export const serviceStatusSchema = services.serviceStatusSchema;

export type ServiceStatus = z.infer<typeof serviceStatusSchema>;

export const WIDGET_SIZES = ["quarter", "third", "half", "two-thirds", "full"] as const;

export const widgetSizeSchema = z.enum(WIDGET_SIZES).catch("full");

export type WidgetSize = z.infer<typeof widgetSizeSchema>;

export const sectionSchema = z.object({ id: z.string(), title: z.string().nullable() });

export type Section = z.infer<typeof sectionSchema>;
