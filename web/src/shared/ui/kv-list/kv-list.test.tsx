import { render, screen } from "@testing-library/react";
import { expect, it } from "vitest";

import { KvList, KvRow } from "./kv-list";

it("pairs labels with values in a description list", () => {
  render(
    <KvList>
      <KvRow label="Port">8080</KvRow>
    </KvList>,
  );
  expect(screen.getByText("Port").tagName).toBe("DT");
  expect(screen.getByText("8080").tagName).toBe("DD");
});
