import { expect, it } from "vitest";

import { DIAGNOSES } from "@/shared/api";

import { diagnosisMessage } from "./diagnosis";

it("every diagnosis code maps to a title and an action key of its own", () => {
  for (const code of DIAGNOSES) {
    expect(diagnosisMessage(code)).toEqual({ title: `diagnosis.${code}.title`, action: `diagnosis.${code}.action` });
  }
});

it("no failure means no message", () => {
  expect(diagnosisMessage(null)).toBeNull();
});
