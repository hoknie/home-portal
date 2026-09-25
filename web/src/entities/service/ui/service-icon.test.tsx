import { fireEvent, render, waitFor } from "@testing-library/react";
import { expect, it, vi } from "vitest";

import { ServiceIcon } from "./service-icon";

it("falls back to a server icon for a name lucide does not have, without logging", () => {
  const error = vi.spyOn(console, "error").mockImplementation(() => undefined);
  const { container } = render(<ServiceIcon name="no-such-icon" />);
  expect(container.querySelector("svg")).toHaveClass("lucide-server");
  expect(error).not.toHaveBeenCalled();
  error.mockRestore();
});

it("shows the icon the portal serves and falls back when the image does not load", () => {
  const { container } = render(<ServiceIcon name={null} source="/api/icons/nas" />);
  const image = container.querySelector("img");
  expect(image?.getAttribute("src")).toContain("/api/icons/nas");
  fireEvent.error(image as HTMLImageElement);
  expect(container.querySelector("img")).toBeNull();
  expect(container.querySelector("svg")).toHaveClass("lucide-server");
});

it("prefers the portal's image over a lucide name, and keeps the name for the fallback", async () => {
  const { container } = render(<ServiceIcon name="film" source="/api/icons/media" />);
  expect(container.querySelector("img")).not.toBeNull();
  fireEvent.error(container.querySelector("img") as HTMLImageElement);
  expect(container.querySelector("img")).toBeNull();
  await waitFor(() => expect(container.querySelector("svg")).toHaveClass("lucide-film"));
});
