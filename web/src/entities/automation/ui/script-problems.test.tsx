import { screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { expect, it } from "vitest";

import { renderWithProviders } from "@/shared/lib/testing";

import { scriptsSchema } from "../model/schema";
import { ScriptProblems } from "./script-problems";

const scripts = scriptsSchema.parse({
  directory: "/srv/portal/scripts",
  exists: true,
  user_id: 501,
  scripts: [
    { path: "open.sh", runnable: false, problem: "open.sh can be written by group or others", code: "writable", concerns: "/srv/portal/scripts/open.sh" },
    { path: "notes.sh", runnable: false, problem: "notes.sh is not executable", code: "not-executable", concerns: "/srv/portal/scripts/notes.sh" },
    { path: "media/restart.sh", runnable: false, problem: "owner", code: "folder-owner", concerns: "/srv/portal/scripts/media" },
    { path: "fine.sh", runnable: true, problem: null, code: null, concerns: null },
  ],
});

it("explains every script that cannot run with the command that fixes it", async () => {
  renderWithProviders(<ScriptProblems scripts={scripts} />);
  await userEvent.click(screen.getByText("Scripts that cannot run: 3"));
  expect(screen.getByText("chmod go-w '/srv/portal/scripts/open.sh'")).toBeInTheDocument();
  expect(screen.getByText("chmod +x '/srv/portal/scripts/notes.sh'")).toBeInTheDocument();
  expect(screen.getAllByText(/#!\/bin\/sh/).length).toBeGreaterThan(0);
  expect(screen.getByText("sudo chown 501 '/srv/portal/scripts/media'")).toBeInTheDocument();
  expect(screen.getByText(/The folder \/srv\/portal\/scripts\/media belongs/)).toBeInTheDocument();
});

it("offers a guide to preparing a script", async () => {
  renderWithProviders(<ScriptProblems scripts={{ ...scripts, scripts: [] }} />);
  await userEvent.click(screen.getByText("How to prepare a script"));
  expect(screen.getByText(/Create the directory \/srv\/portal\/scripts/)).toBeInTheDocument();
  expect(screen.getByText(/uid 501/)).toBeInTheDocument();
});
