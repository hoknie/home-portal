import { describe, expect, it } from "vitest";

import { fromArgs, toArgs } from "./arguments";
import { parseHeader } from "./header";

const declared = parseHeader("# @arg service <text> Service id\n# @arg --retries <number=3> Tries\n# @arg --force Skip the check\n").arguments;

describe("mapping declared arguments onto args", () => {
  it("writes positionals first, then filled options, and a flag alone", () => {
    expect(
      toArgs(declared, {
        service: "{{service.id}}",
        "--retries": "",
        "--force": true,
      }),
    ).toEqual(["{{service.id}}", "--force"]);
    expect(toArgs(declared, { service: "nas", "--retries": "5" })).toEqual(["nas", "--retries", "5"]);
  });

  it("reads the same args back", () => {
    expect(fromArgs(declared, ["{{service.id}}", "--force"])).toEqual({
      service: "{{service.id}}",
      "--force": true,
    });
    expect(fromArgs(declared, ["nas", "--retries", "5", "--force"])).toEqual({
      service: "nas",
      "--retries": "5",
      "--force": true,
    });
  });

  it("gives up on args that do not follow the declared form", () => {
    expect(fromArgs(declared, ["--force", "{{service.id}}"])).toBeNull();
    expect(fromArgs(declared, ["nas", "--unknown"])).toBeNull();
    expect(fromArgs(declared, ["nas", "--retries"])).toBeNull();
    expect(fromArgs([], ["nas"])).toBeNull();
  });

  it("keeps an empty optional positional only when a later one has a value", () => {
    const two = parseHeader("# @arg host?\n# @arg port?\n").arguments;
    expect(toArgs(two, { host: "", port: "8080" })).toEqual(["", "8080"]);
    expect(toArgs(two, { host: "nas", port: "" })).toEqual(["nas"]);
  });
});
