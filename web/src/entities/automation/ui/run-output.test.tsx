import { screen } from "@testing-library/react";
import { expect, it } from "vitest";

import { renderWithProviders } from "@/shared/lib/testing";

import { RunOutput } from "./run-output";

it("draws curl's progress meter as a terminal would, one line per line", () => {
  const tail = "  % Total\n\r  0  0\r 50  1M\r100  2M\n\u001b[32mok\u001b[0m\n";
  renderWithProviders(<RunOutput label="Errors" output={{ tail, bytes: tail.length, truncated: false }} />);
  expect(screen.getByText("Errors").nextElementSibling?.textContent).toBe("  % Total\n100  2M\nok\n");
});

it("notes a truncated output and says when there is none", () => {
  renderWithProviders(
    <>
      <RunOutput label="Output" output={{ tail: "end\n", bytes: 200_000, truncated: true }} />
      <RunOutput label="Errors" output={{ tail: "", bytes: 0, truncated: false }} />
    </>,
  );
  expect(screen.getByText(/^The last 64 KiB of 200,?000 bytes$/)).toBeInTheDocument();
  expect(screen.getByText("empty")).toBeInTheDocument();
});
