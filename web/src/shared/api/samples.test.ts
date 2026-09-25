import { expect, it } from "vitest";

import { apiSamples } from "./samples";
import { fieldErrorsSchema } from "./schemas";

it("the field errors sample parses with the schema the client uses for 422", () => {
  expect(fieldErrorsSchema.parse(apiSamples.fieldErrors).errors.map((error) => error.field)).toEqual(["id", "url"]);
});
