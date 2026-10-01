import { existsSync, mkdirSync, readFileSync, readdirSync, rmSync, writeFileSync } from "node:fs";
import { join } from "node:path";

import { generate } from "./generate.mjs";

const WEB = new URL("../..", import.meta.url).pathname.replace(/^\/@fs\//, "/");
export const SCHEMAS = join(WEB, "src/shared/api/json-schemas");
export const GENERATED = join(WEB, "src/shared/api/generated");
const SUFFIX = ".schema.json";

export function answers() {
  return existsSync(SCHEMAS)
    ? readdirSync(SCHEMAS)
        .filter((name) => name.endsWith(SUFFIX))
        .map((name) => name.slice(0, -SUFFIX.length))
        .sort()
    : [];
}

export function generated(answer) {
  return generate(JSON.parse(readFileSync(join(SCHEMAS, `${answer}${SUFFIX}`), "utf8")), answer);
}

export function index(names) {
  return names.map((answer) => `export * as ${answer.replace(/-([a-z0-9])/g, (_, letter) => letter.toUpperCase())} from "./${answer}";`).join("\n") + "\n";
}

export function writeAll() {
  const names = answers();
  mkdirSync(GENERATED, { recursive: true });
  const wanted = new Set([...names.map((answer) => `${answer}.ts`), "index.ts"]);
  for (const name of readdirSync(GENERATED)) {
    if (name.endsWith(".ts") && !name.endsWith(".test.ts") && !wanted.has(name)) {
      rmSync(join(GENERATED, name));
    }
  }
  for (const answer of names) {
    writeFileSync(join(GENERATED, `${answer}.ts`), generated(answer));
  }
  writeFileSync(join(GENERATED, "index.ts"), index(names));
  return names;
}
