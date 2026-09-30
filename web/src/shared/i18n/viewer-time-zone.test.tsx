import { render, screen, waitFor } from "@testing-library/react";
import { useFormatter, useTimeZone } from "next-intl";
import { afterEach, expect, it, vi } from "vitest";

import { TestIntl } from "./test-intl";
import { ViewerTimeZone } from "./viewer-time-zone";

afterEach(() => {
  vi.restoreAllMocks();
});

function Shown() {
  const format = useFormatter();
  return (
    <p>
      {useTimeZone()} {format.dateTime(new Date("2026-09-30T12:00:00Z"), { hour: "2-digit", minute: "2-digit", hourCycle: "h23" })}
    </p>
  );
}

it("times follow the time zone of the viewer's browser", async () => {
  vi.spyOn(Intl.DateTimeFormat.prototype, "resolvedOptions").mockReturnValue({ ...new Intl.DateTimeFormat().resolvedOptions(), timeZone: "Europe/Madrid" });
  render(
    <TestIntl>
      <ViewerTimeZone>
        <Shown />
      </ViewerTimeZone>
    </TestIntl>,
  );
  await waitFor(() => expect(screen.getByText(/Europe\/Madrid 14:00/)).toBeInTheDocument());
});
