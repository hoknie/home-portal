import { render, screen } from "@testing-library/react";
import { expect, it } from "vitest";

import { weatherSchema, widgetDataSchema } from "@/entities/widget";
import { apiSamples } from "@/shared/api";
import { TestIntl } from "@/shared/i18n";

import { WeatherWidget } from "./weather-widget";

const data = weatherSchema.parse(widgetDataSchema.parse(apiSamples.widgetWeather).data);

function renderWidget(reading = data) {
  render(
    <TestIntl>
      <WeatherWidget data={reading} />
    </TestIntl>,
  );
}

it("shows the current temperature, the condition in words and how it feels", () => {
  renderWidget();
  expect(screen.getByText("12°C")).toBeInTheDocument();
  expect(screen.getByText("partly cloudy")).toBeInTheDocument();
  expect(screen.getByText("feels like 11°C")).toBeInTheDocument();
  expect(screen.getByText("humidity 78%")).toBeInTheDocument();
});

it("lists every day with its highest and lowest temperature", () => {
  renderWidget();
  expect(screen.getByText("14°C")).toBeInTheDocument();
  expect(screen.getByText("at night 8°C")).toBeInTheDocument();
  expect(screen.getByText("16°C")).toBeInTheDocument();
  expect(screen.getByText("at night 7°C")).toBeInTheDocument();
});

it("falls back to a cloud for a condition it has no icon for", () => {
  const { container } = render(
    <TestIntl>
      <WeatherWidget data={{ ...data, current: { ...data.current, condition: "unknown" } }} />
    </TestIntl>,
  );
  expect(screen.getByText("no data")).toBeInTheDocument();
  expect(container.querySelector("svg")).toHaveClass("lucide-cloudy");
});
