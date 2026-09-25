import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { expect, it } from "vitest";

import { Tooltip, TooltipContent, TooltipProvider, TooltipTrigger } from "./tooltip";

it("shows its content on hover", async () => {
  render(
    <TooltipProvider delayDuration={0}>
      <Tooltip>
        <TooltipTrigger>hover me</TooltipTrigger>
        <TooltipContent>hint</TooltipContent>
      </Tooltip>
    </TooltipProvider>,
  );
  await userEvent.hover(screen.getByText("hover me"));
  expect((await screen.findAllByText("hint")).length).toBeGreaterThan(0);
});
