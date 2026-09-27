import { describe, expect, it } from "vitest";

import { apiSamples } from "@/shared/api";

import { enabledModules, modulesSchema, switchLock } from "./schema";

const modules = modulesSchema.parse(apiSamples.modules);
const byName = (name: string) => modules.modules.find((module) => module.name === name)!;

describe("modules", () => {
  it("the sample parses with every module in order", () => {
    expect(modules.modules.map((module) => module.name)).toEqual(["proxy", "dns", "automations", "webhooks", "users"]);
    expect([...enabledModules(modules)]).toEqual(["proxy", "dns", "automations", "users"]);
  });

  it("an enabled module that another needs is locked by it", () => {
    expect(switchLock(byName("proxy"), modules)).toEqual({ kind: "required-by", modules: ["dns"] });
    expect(switchLock(byName("automations"), modules)).toBeNull();
  });

  it("a disabled module whose requirement is off is locked by that requirement", () => {
    const off = modulesSchema.parse({
      modules: modules.modules.map((module) => (module.name === "automations" ? { ...module, enabled: false } : module)),
    });
    expect(switchLock(off.modules[3], off)).toEqual({ kind: "requires", modules: ["automations"] });
    expect(switchLock(byName("webhooks"), modules)).toBeNull();
  });

  it("no modules loaded means none is enabled", () => {
    expect(enabledModules(undefined).size).toBe(0);
  });
});
