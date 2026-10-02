import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { expect, it } from "vitest";

import { Tabs, TabsContent, TabsList, TabsTrigger } from "./tabs";

it("shows the content of the chosen tab and moves between tabs by keyboard", async () => {
  render(
    <Tabs defaultValue="general">
      <TabsList aria-label="Settings">
        <TabsTrigger value="general">General</TabsTrigger>
        <TabsTrigger value="look">Look</TabsTrigger>
      </TabsList>
      <TabsContent value="general">Title field</TabsContent>
      <TabsContent value="look">Surface choices</TabsContent>
    </Tabs>,
  );
  expect(screen.getByRole("tab", { name: "General" })).toHaveAttribute("aria-selected", "true");
  expect(screen.getByText("Title field")).toBeInTheDocument();
  await userEvent.click(screen.getByRole("tab", { name: "Look" }));
  expect(screen.getByText("Surface choices")).toBeInTheDocument();
  screen.getByRole("tab", { name: "Look" }).focus();
  await userEvent.keyboard("{ArrowLeft}");
  expect(screen.getByRole("tab", { name: "General" })).toHaveFocus();
});
