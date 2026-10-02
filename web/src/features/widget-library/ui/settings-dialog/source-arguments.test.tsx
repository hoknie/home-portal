import { screen, within } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { useState } from "react";
import { expect, it, vi } from "vitest";

import { renderWithProviders } from "@/shared/lib/testing";

import { ScriptArguments, WorkflowInputs, type ScriptArgumentsProps, type WorkflowInputsProps } from "./source-arguments";

const argument = { choices: [] as string[], default: null as string | null, description: null as string | null, option: false, required: true, type: "text" };

const DISK_CONTROL = {
  path: "disk-control.sh",
  runnable: true,
  problem: null,
  description: "Disk eject/inject",
  argument_problems: [],
  arguments: [
    { ...argument, name: "disk_id", description: "Disk id from diskutil list" },
    { ...argument, name: "action", choices: ["inject", "eject"] },
    { ...argument, name: "--timeout", option: true, required: false, type: "number", default: "40" },
  ],
} as unknown as ScriptArgumentsProps["script"];

function Script({ spy }: { spy: (args: string[]) => void }) {
  const [args, setArgs] = useState<string[]>([]);
  return (
    <ScriptArguments
      script={DISK_CONTROL}
      args={args}
      onChange={(next) => {
        spy(next);
        setArgs(next);
      }}
    />
  );
}

it("reads the arguments a script's header declares as fields of their kind", async () => {
  const spy = vi.fn();
  renderWithProviders(<Script spy={spy} />);
  await userEvent.type(screen.getByLabelText(/disk_id/), "disk4");
  await userEvent.click(within(screen.getByRole("group", { name: "Choices" })).getByRole("button", { name: "eject" }));
  expect(screen.getByLabelText(/--timeout/)).toHaveAttribute("placeholder", "40");
  expect(spy).toHaveBeenLastCalledWith(["disk4", "eject"]);
});

it("keeps a free list of arguments for a script without a header", () => {
  renderWithProviders(<ScriptArguments script={{ ...DISK_CONTROL, arguments: [] } as ScriptArgumentsProps["script"]} args={["-v"]} onChange={vi.fn()} />);
  expect(screen.getByText("-v")).toBeInTheDocument();
  expect(screen.queryByLabelText(/disk_id/)).not.toBeInTheDocument();
});

const WORKFLOW = {
  inputs: [
    { name: "retries", type: "number", default: 3, description: "Tries" },
    { name: "hosts", type: "list", default: null, description: null },
  ],
} as unknown as WorkflowInputsProps["workflow"];

function Inputs({ spy }: { spy: (values: Record<string, unknown>) => void }) {
  const [values, setValues] = useState<Record<string, unknown>>({});
  return (
    <WorkflowInputs
      workflow={WORKFLOW}
      values={values}
      errors={{}}
      onChange={(next) => {
        spy(next);
        setValues(next);
      }}
    />
  );
}

it("offers each workflow input by its type, with its default, and takes no text for a number", async () => {
  const spy = vi.fn();
  renderWithProviders(<Inputs spy={spy} />);
  const retries = screen.getByRole("spinbutton", { name: "retries" });
  expect(retries).toHaveAttribute("placeholder", "3");
  expect(screen.getByText("Tries · Default: 3")).toBeInTheDocument();
  await userEvent.type(retries, "three");
  expect(retries).toHaveValue(null);
  await userEvent.clear(retries);
  await userEvent.type(retries, "5");
  expect(spy).toHaveBeenLastCalledWith({ retries: 5 });
  await userEvent.click(screen.getByRole("button", { name: "Add a value" }));
  await userEvent.type(screen.getByRole("textbox", { name: "Value 1" }), "nas");
  expect(spy).toHaveBeenLastCalledWith({ retries: 5, hosts: ["nas"] });
});
