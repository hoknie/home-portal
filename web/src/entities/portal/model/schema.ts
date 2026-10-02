import type { z } from "zod";

import { generated } from "@/shared/api";

export const publicServiceSchema = generated.publicPortal.publicServiceSchema;

export type PublicService = z.infer<typeof publicServiceSchema>;

export const publicWidgetSchema = generated.publicPortal.publicWidgetSchema;

export type PublicWidget = z.infer<typeof publicWidgetSchema>;

export const portalSchema = generated.publicPortal.portalResponseSchema;

export type Portal = z.infer<typeof portalSchema>;

export const failureReportSchema = generated.failureReport.failureReportResponseSchema;

export type FailureReport = z.infer<typeof failureReportSchema>;

export type Problem = z.infer<typeof generated.failureReport.problemResponseSchema>;
