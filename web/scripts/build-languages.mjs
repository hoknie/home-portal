import { spawnSync } from "node:child_process";
import { createHash } from "node:crypto";
import { cpSync, existsSync, mkdirSync, readFileSync, readdirSync, renameSync, rmSync, statSync } from "node:fs";
import { join, relative } from "node:path";

const WEB = new URL("..", import.meta.url).pathname;
const MESSAGES = join(WEB, "src/shared/i18n/messages");
const OUT = join(WEB, "out");
const STAGING = join(WEB, ".languages");
const NEXT = "_next";

const languages = readdirSync(MESSAGES)
  .filter((name) => name.endsWith(".json"))
  .map((name) => name.slice(0, -".json".length))
  .sort();

function files(directory) {
  return readdirSync(directory, { withFileTypes: true }).flatMap((entry) => {
    const path = join(directory, entry.name);
    return entry.isDirectory() ? files(path) : [path];
  });
}

function digest(path) {
  return createHash("sha256").update(readFileSync(path)).digest("hex");
}

function size(directory) {
  return files(directory).reduce((total, path) => total + statSync(path).size, 0);
}

const started = Date.now();
rmSync(STAGING, { recursive: true, force: true });
mkdirSync(STAGING, { recursive: true });
for (const language of languages) {
  const began = Date.now();
  const build = spawnSync("next", ["build"], {
    cwd: WEB,
    stdio: "inherit",
    shell: process.platform === "win32",
    env: { ...process.env, PORTAL_LANGUAGE: language },
  });
  if (build.status !== 0) {
    console.error(`next build failed for ${language}`);
    process.exit(build.status ?? 1);
  }
  renameSync(OUT, join(STAGING, language));
  console.log(`built ${language} in ${((Date.now() - began) / 1000).toFixed(1)} s`);
}

rmSync(OUT, { recursive: true, force: true });
mkdirSync(join(OUT, NEXT), { recursive: true });
const seen = new Map();
let shared = 0;
for (const language of languages) {
  const tree = join(STAGING, language);
  for (const entry of readdirSync(tree)) {
    if (entry !== NEXT) {
      cpSync(join(tree, entry), join(OUT, language, entry), { recursive: true });
    }
  }
  for (const path of files(join(tree, NEXT))) {
    const name = relative(tree, path);
    const hash = digest(path);
    const earlier = seen.get(name);
    if (earlier === undefined) {
      seen.set(name, hash);
      cpSync(path, join(OUT, name));
    } else if (earlier === hash) {
      shared += 1;
    } else {
      console.error(`${name} differs between language builds; hashed assets must not collide`);
      process.exit(1);
    }
  }
  const lang = readFileSync(join(OUT, language, "index.html"), "utf8").match(/<html[^>]*\slang="([^"]+)"/)?.[1];
  if (lang !== language) {
    console.error(`${language}/index.html declares lang="${lang}"`);
    process.exit(1);
  }
}
rmSync(STAGING, { recursive: true, force: true });

const megabytes = (bytes) => (bytes / 1024 / 1024).toFixed(1);
console.log(`languages: ${languages.join(", ")}`);
console.log(`_next files shared between builds: ${shared}; distinct: ${seen.size}; collisions: 0`);
console.log(`_next: ${megabytes(size(join(OUT, NEXT)))} MB; whole out: ${megabytes(size(OUT))} MB`);
console.log(`built in ${((Date.now() - started) / 1000).toFixed(1)} s`);
if (!existsSync(join(OUT, languages[0], "index.html"))) {
  process.exit(1);
}
