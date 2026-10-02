import { z } from "zod";

import * as dashboard from "./generated/dashboard";
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

export const sectionSchema = dashboard.sectionViewSchema;

export type Section = z.infer<typeof sectionSchema>;

export type WidgetHeight = dashboard.WidgetHeight;

export type Appearance = dashboard.ResolvedAppearance;

export type SectionAppearance = dashboard.ResolvedSectionAppearance;

export const SURFACES = dashboard.surfaceSchema.unwrap().options;

export const ACCENTS = dashboard.accentSchema.unwrap().options;

export const PADDINGS = dashboard.paddingSchema.unwrap().options;

export const ALIGNS = dashboard.alignSchema.unwrap().options;

export const TITLE_VISIBILITIES = dashboard.titleVisibilitySchema.unwrap().options;

export const SECTION_SURFACES = dashboard.sectionSurfaceSchema.unwrap().options;

export const DEFAULT_APPEARANCE: Appearance = { surface: "card", accent: "neutral", title: "shown", padding: "normal", align: "start" };

export const DEFAULT_SECTION_APPEARANCE: SectionAppearance = { title: "shown", surface: "none" };

export const GRID_COLUMNS = 12;

export const LARGEST_ROWS = 8;
