import { RequestError, request } from "@/shared/api";
import { api } from "@/shared/config";

import { type FailureReport, failureReportSchema } from "../model/schema";

export type FailureState = { kind: "failed"; report: FailureReport } | { kind: "running" };

export const RUNNING_STATUS = 404;

export async function fetchFailure(): Promise<FailureState> {
  try {
    const { data } = await request(api.portalFailure, { schema: failureReportSchema, redirectOnUnauthorized: false });
    return { kind: "failed", report: data };
  } catch (error) {
    if (error instanceof RequestError && error.status === RUNNING_STATUS) {
      return { kind: "running" };
    }
    throw error;
  }
}
