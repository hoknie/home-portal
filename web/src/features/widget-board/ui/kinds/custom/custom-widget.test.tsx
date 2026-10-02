import { screen, within } from "@testing-library/react";
import { expect, it, vi } from "vitest";

import { type CustomWidgetData, customWidgetSchema } from "@/entities/widget";
import { apiSamples } from "@/shared/api";
import { renderWithProviders } from "@/shared/lib/testing";
import { WidgetFrame } from "@/shared/ui/widget-frame";

import { CustomWidget } from "./custom-widget";

vi.mock("next/navigation", () => ({ useRouter: () => ({ push: vi.fn(), replace: vi.fn() }) }));

const sample = customWidgetSchema.parse(apiSamples.customWidget);

function draw(data: CustomWidgetData, scope: "private" | "public" = "private") {
  return renderWithProviders(<CustomWidget data={data} widget="disks" scope={scope} />);
}

it("draws every kind of block the portal renders, with its tone", () => {
  const { container } = draw(sample);
  expect(screen.getByText("Free")).toBeInTheDocument();
  expect(container.querySelector('[data-block="stat"]')).toHaveAttribute("data-tone", "ok");
  expect(screen.getByRole("meter", { name: "Used" })).toHaveAttribute("aria-valuenow", "93");
  expect(container.querySelector('[data-block="progress"]')).toHaveAttribute("data-tone", "danger");
  expect(screen.getByText("failed")).toHaveAttribute("data-tone", "danger");
  expect(screen.getByText("phone")).toBeInTheDocument();
  expect(screen.getByText("13 more items")).toBeInTheDocument();
  expect(screen.getByRole("columnheader", { name: "Address" })).toBeInTheDocument();
  expect(screen.getByText("Uptime")).toBeInTheDocument();
  expect(container.querySelector('[data-block="row"]')).not.toBeNull();
  expect(container.querySelector('[data-block="divider"]')).not.toBeNull();
  expect(screen.getByRole("button", { name: "Restart" })).toBeInTheDocument();
});

it("shows text from data as text and never builds an element from it", () => {
  const { container } = draw({ blocks: [{ kind: "text", align: null, valign: null, text: "<img src=x onerror=alert(1)>", size: "normal", weight: "normal", muted: false, tone: "neutral" }] });
  expect(screen.getByText("<img src=x onerror=alert(1)>")).toBeInTheDocument();
  expect(container.querySelector("img")).toBeNull();
});

it("markdown keeps web links in a new tab and drops scripts, raw HTML and images", () => {
  const { container } = draw({ blocks: [{ kind: "markdown", align: null, valign: null, text: "[open](javascript:alert(1)) [nas](https://nas.home.lan) <b>raw</b> ![x](https://x.example/a.png)" }] });
  const markdown = container.querySelector('[data-block="markdown"]') as HTMLElement;
  const links = within(markdown).getAllByRole("link");
  expect(links).toHaveLength(1);
  expect(links[0]).toHaveAttribute("href", "https://nas.home.lan");
  expect(links[0]).toHaveAttribute("target", "_blank");
  expect(within(markdown).getByText("open").closest("a")).toBeNull();
  expect(markdown.querySelector("b")).toBeNull();
  expect(markdown.querySelector("img")).toBeNull();
});

it("a public widget draws no buttons", () => {
  draw(sample, "public");
  expect(screen.queryByRole("button")).toBeNull();
  expect(screen.queryByRole("link", { name: "Open" })).toBeNull();
});

it("a row shares its width by parts, holds a column, and each block sits where its align puts it", () => {
  const { container } = draw(sample);
  const row = container.querySelector('[data-block="row"]') as HTMLElement;
  expect(row.style.getPropertyValue("--row-columns")).toBe("minmax(0,4fr) minmax(0,8fr)");
  expect(row.className).toContain("sm:items-center");
  const column = row.querySelector(':scope > [data-block="column"]') as HTMLElement;
  expect(column.querySelector('[data-block="row"]')).not.toBeNull();
  expect(within(column).getByText("Backup **3 h** ago").parentElement).toHaveAttribute("data-align", "end");
  expect(screen.getByText("failed").parentElement).toHaveAttribute("data-align", "center");
});

it("a block without align takes the widget's, so a centred widget centres its number, bar and button", () => {
  const blocks = sample.blocks.filter((block) => block.kind === "stat" || block.kind === "progress");
  const { container } = renderWithProviders(
    <WidgetFrame title="Disks" appearance={{ surface: "card", accent: "neutral", title: "shown", padding: "normal", align: "center" }}>
      <CustomWidget data={{ blocks: [...blocks, { kind: "text", align: "start", valign: null, text: "left", size: "normal", weight: "normal", muted: false, tone: "neutral" }] }} widget="disks" scope="private" />
    </WidgetFrame>,
  );
  const placed = [...container.querySelectorAll("[data-custom-widget] > [data-align]")].map((element) => element.getAttribute("data-align"));
  expect(placed).toEqual(["center", "center", "start"]);
  expect(container.querySelector("section")).toHaveClass("items-center", "text-center");
});

it("a block placed lower in its row and a column centring its blocks", () => {
  const leaf = { kind: "text" as const, align: null, size: "normal" as const, weight: "normal" as const, muted: false, tone: "neutral" as const };
  const { container } = draw({
    blocks: [
      {
        kind: "row",
        align: "stretch",
        gap: "normal",
        widths: null,
        blocks: [
          { ...leaf, text: "low", valign: "end" },
          { kind: "column", align: null, valign: "center", gap: "normal", blocks: [{ ...leaf, text: "mid", valign: null }] },
        ],
      },
    ],
  });
  expect(screen.getByText("low").parentElement).toHaveClass("sm:self-end");
  expect(container.querySelector('[data-block="column"]')).toHaveClass("content-center", "h-full");
});
