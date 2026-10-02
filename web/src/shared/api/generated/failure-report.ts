import { z } from "zod";

export const problemResponseSchema = z.object({ "field": z.string().nullable(), "file": z.string().nullable(), "message": z.string() });

export type ProblemResponse = z.infer<typeof problemResponseSchema>;

export const failureReportResponseSchema = z.object({ "checked": z.string(), "details": z.boolean(), "problems": z.array(problemResponseSchema).nullable(), "since": z.string() });

export type FailureReportResponse = z.infer<typeof failureReportResponseSchema>;

export const schema = failureReportResponseSchema;
