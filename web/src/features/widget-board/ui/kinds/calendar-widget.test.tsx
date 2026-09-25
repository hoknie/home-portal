import { render, screen } from "@testing-library/react";
import { expect, it } from "vitest";

import { calendarSchema, widgetDataSchema } from "@/entities/widget";
import { apiSamples } from "@/shared/api";
import { TestIntl } from "@/shared/i18n";

import { CalendarWidget } from "./calendar-widget";

const data = calendarSchema.parse(widgetDataSchema.parse(apiSamples.widgetCalendar).data);

function renderWidget(reading = data) {
  render(
    <TestIntl>
      <CalendarWidget data={reading} />
    </TestIntl>,
  );
}

it("groups the events by day and names their time and place", () => {
  renderWidget();
  const days = screen.getAllByRole("listitem").map((item) => item.textContent);
  expect(days[0]).toContain("22");
  expect(days[0]).toContain("Pick up a parcel");
  expect(days[0]).toContain("Post office");
  expect(days[1]).toContain("23");
  expect(screen.getByText("09:00 AM")).toBeInTheDocument();
});

it("marks an event whose repeat rule the portal cannot expand", () => {
  renderWidget();
  const repeat = screen.getByText("repeats");
  expect(repeat.closest("[title]")).toHaveAttribute(
    "title",
    "This repeat rule is not supported: only the next event is shown.",
  );
});

it("says so when there is nothing ahead", () => {
  renderWidget({ events: [] });
  expect(screen.getByText("No upcoming events")).toBeInTheDocument();
});
