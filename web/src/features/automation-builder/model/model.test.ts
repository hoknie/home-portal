import { describe, expect, it } from "vitest";

import { automationsSchema } from "@/entities/automation";
import { apiSamples } from "@/shared/api";

import { presetCron } from "./cron";
import { automationFormSchema, emptyAutomationForm, foreignFilters, formOf, requestOf } from "./form";
import { insertAt, renderTemplate, unknownPlaceholders } from "./placeholders";
import { commandLineOf, variableOf } from "./preview";
import { formPathOf } from "./server-errors";

const values = { "service.id": "nas", "status.error": "{{service.name}}" };

describe("placeholders", () => {
  it.each([
    ["{{service.id}}", "nas"],
    ["--id={{service.id}}!", "--id=nas!"],
    ["{{ nope", "{{ nope"],
    ["{x}", "{x}"],
    ["{{}}", "{{}}"],
    ["{{{service.id}}}", "{nas}"],
    ["{{Service.Id}}", "{{Service.Id}}"],
    ["{{service.name}}", "{{service.name}}"],
    ["{{service.id}}{{service.id}}", "nasnas"],
  ])("renders %s as %s", (template, expected) => {
    expect(renderTemplate(template, values)).toBe(expected);
  });

  it("a placeholder may hold digits after the first letter", () => {
    expect(renderTemplate("{{webhook.build2}}", { "webhook.build2": "7" })).toBe("7");
    expect(renderTemplate("{{webhook.2build}}", { "webhook.2build": "7" })).toBe("{{webhook.2build}}");
  });

  it("never takes a property every object has for a field", () => {
    expect(renderTemplate("{{constructor}}", values)).toBe("{{constructor}}");
  });

  it("never expands a value a second time", () => {
    expect(renderTemplate("{{status.error}}", values)).toBe("{{service.name}}");
  });

  it("reports a field the event does not have", () => {
    expect(unknownPlaceholders("{{service.id}} {{service.name}} {{ no }}", ["service.id"])).toEqual(["service.name"]);
  });

  it("inserts a token at the cursor or at the end", () => {
    expect(insertAt("--", 2, "{{service.id}}")).toEqual({ text: "--{{service.id}}", cursor: 16 });
    expect(insertAt("ab", 1, "X")).toEqual({ text: "aXb", cursor: 2 });
    expect(insertAt("ab", null, "X").text).toBe("abX");
  });
});

describe("preview", () => {
  it("quotes each argument apart and names the variables", () => {
    expect(commandLineOf("restart.sh", ["--service", "{{service.id}}", "it's"], { "service.id": "jellyfin" })).toBe(
      "restart.sh '--service' 'jellyfin' 'it'\\''s'",
    );
    expect(variableOf("sign_in.reason")).toBe("PORTAL_SIGN_IN_REASON");
  });
});

describe("cron presets", () => {
  it("build the expression from the chosen time and day", () => {
    expect(presetCron("hourly", "03:00", 1)).toBe("0 * * * *");
    expect(presetCron("daily", "03:00", 1)).toBe("0 3 * * *");
    expect(presetCron("weekly", "22:30", 5)).toBe("30 22 * * 5");
    expect(presetCron("monthly", "07:05", 1)).toBe("5 7 1 * *");
  });
});

describe("form", () => {
  const fieldsOf = (event: string) => (event === "portal.started" ? ["portal.address"] : ["service.id", "status.to"]);

  it("a sample automation round-trips into the same request", () => {
    const automation = automationsSchema.parse(apiSamples.automations).automations[0];
    expect(requestOf(formOf(automation), ["services", "from", "to", "from_unknown"])).toEqual({
      id: "restart-media",
      title: "Restart Jellyfin when it goes down",
      enabled: true,
      tags: ["media", "night"],
      cooldown_seconds: 300,
      when: { event: "service.status-changed", services: ["jellyfin"], to: ["down", "unreadable"] },
      run: { script: "restart.sh", args: ["--", "{{service.id}}"], timeout_seconds: 120 },
    });
  });

  it("a request carries only the filters of its event", () => {
    const form = { ...emptyAutomationForm, event: "user.signed-in" as const, to: ["down"], users: ["admin"] };
    expect(requestOf(form, ["users", "environments"]).when).toEqual({ event: "user.signed-in", users: ["admin"] });
    expect(foreignFilters(["users", "environments"])).toEqual(["services", "from", "to", "from_unknown", "webhooks", "cron"]);
  });

  it("mirrors the server's rules", () => {
    const schema = automationFormSchema(fieldsOf);
    const valid = { ...emptyAutomationForm, id: "a", title: "A", script: "a.sh", args: [{ value: "{{service.id}}" }] };
    expect(schema.safeParse(valid).success).toBe(true);
    const issues = (form: typeof valid) => (schema.safeParse(form).error?.issues ?? []).map((issue) => issue.path.join("."));
    expect(issues({ ...valid, id: "Bad Id" })).toEqual(["id"]);
    expect(issues({ ...valid, id: "runs" })).toEqual(["id"]);
    expect(issues({ ...valid, script: "../home-portal.toml" })).toEqual(["script"]);
    expect(issues({ ...valid, script: "/bin/sh" })).toEqual(["script"]);
    expect(issues({ ...valid, timeout_seconds: 3601 })).toEqual(["timeout_seconds"]);
    expect(issues({ ...valid, event: "schedule" })).toEqual(["cron"]);
    expect(issues({ ...valid, from: ["unknown"] })).toEqual(["from"]);
    expect(issues({ ...valid, event: "portal.started" })).toEqual(["args.0.value"]);
  });

  it("server errors land on the form's fields", () => {
    expect(formPathOf("when.to")).toBe("to");
    expect(formPathOf("when.cron")).toBe("cron");
    expect(formPathOf("run.args[2]")).toBe("args.2.value");
    expect(formPathOf("run.script")).toBe("script");
    expect(formPathOf("id")).toBe("id");
  });
});
