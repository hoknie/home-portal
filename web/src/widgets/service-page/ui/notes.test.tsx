import { render, screen } from "@testing-library/react";
import { expect, it } from "vitest";

import { Notes } from "./notes";

it("renders markdown, opens links in a new tab and shows raw html as text", () => {
  const { container } = render(<Notes text={"**Restart:** `docker restart jellyfin`\n\n[Docs](https://jellyfin.org)\n\n<img src=x onerror=alert(1)>"} />);
  expect(screen.getByText("Restart:").tagName).toBe("STRONG");
  expect(screen.getByText("docker restart jellyfin").tagName).toBe("CODE");
  expect(screen.getByRole("link", { name: "Docs" })).toHaveAttribute("target", "_blank");
  expect(container.querySelector("img")).toBeNull();
});
