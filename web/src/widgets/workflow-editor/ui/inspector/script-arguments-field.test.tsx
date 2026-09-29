import { fireEvent, screen, waitFor, within } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { afterEach, beforeAll, expect, it, vi } from "vitest";

import { parseHeader } from "@/entities/script";
import { jsonResponse } from "@/shared/lib/testing";

import { inspector, node, openEditor, pressOnCanvas, sampleWorkflows, sentBody, stubCanvasDom, withSteps } from "../testing-support";

const header = parseHeader("# @description Restart a container\n# @arg service <text> Service id\n# @arg --force Skip the check\n");
const scripts = [
  {
    path: "media/restart.sh",
    runnable: true,
    problem: null,
    description: header.description,
    arguments: header.arguments,
    argument_problems: [],
  },
];

beforeAll(() => {
  stubCanvasDom();
});

afterEach(() => {
  vi.unstubAllGlobals();
});

it("a script step offers the script's declared arguments as fields and writes the args", async () => {
  const fetch = vi.fn(async () => jsonResponse(sampleWorkflows[1]));
  vi.stubGlobal("fetch", fetch);
  const { onSaved } = openEditor(withSteps([{ id: "run", kind: "script", script: "media/restart.sh" }]), { scripts });
  pressOnCanvas(await node("run"));
  expect(within(inspector()).getByText("Restart a container")).toBeInTheDocument();
  fireEvent.change(within(inspector()).getByRole("combobox", { name: "service" }), { target: { value: "nas" } });
  await userEvent.click(within(inspector()).getByRole("switch", { name: "--force" }));
  await userEvent.click(screen.getByRole("button", { name: "Save" }));
  await waitFor(() => expect(onSaved).toHaveBeenCalled());
  expect(sentBody(fetch).steps[0]).toMatchObject({ args: ["nas", "--force"] });
});

it("args that do not fit the declared form stay a list", async () => {
  openEditor(
    withSteps([
      {
        id: "run",
        kind: "script",
        script: "media/restart.sh",
        args: ["--force", "nas"],
      },
    ]),
    { scripts },
  );
  pressOnCanvas(await node("run"));
  expect(within(inspector()).getByText("These arguments do not follow what the script declares, so they are kept as a list.")).toBeInTheDocument();
  expect(within(inspector()).queryByRole("switch", { name: "--force" })).not.toBeInTheDocument();
});
