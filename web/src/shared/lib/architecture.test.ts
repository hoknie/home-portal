import { existsSync, readdirSync, readFileSync, statSync } from "node:fs";
import { join, relative } from "node:path";

import ts from "typescript";
import { describe, expect, it } from "vitest";

const SOURCE_ROOT = join(process.cwd(), "src");
const FILE_LINES = 400;
const FUNCTION_LINES = 300;
const KIT = "shared/ui/";

type Source = { path: string; text: string };

function walk(directory: string): string[] {
  return readdirSync(directory).flatMap((name) => {
    const path = join(directory, name);
    return statSync(path).isDirectory() ? walk(path) : [path];
  });
}

function sources(): Source[] {
  return walk(SOURCE_ROOT)
    .filter((path) => /\.(ts|tsx)$/.test(path))
    .map((path) => ({ path: relative(SOURCE_ROOT, path), text: readFileSync(path, "utf8") }));
}

function parse(source: Source) {
  const kind = source.path.endsWith(".tsx") ? ts.ScriptKind.TSX : ts.ScriptKind.TS;
  return ts.createSourceFile(source.path, source.text, ts.ScriptTarget.Latest, true, kind);
}

function commentLines(source: Source): number[] {
  const file = parse(source);
  const found = new Set<number>();
  const record = (ranges: ts.CommentRange[] | undefined) => {
    for (const range of ranges ?? []) {
      found.add(file.getLineAndCharacterOfPosition(range.pos).line + 1);
    }
  };
  const visit = (node: ts.Node) => {
    record(ts.getLeadingCommentRanges(source.text, node.getFullStart()));
    record(ts.getTrailingCommentRanges(source.text, node.getEnd()));
    for (const child of node.getChildren(file)) {
      visit(child);
    }
  };
  visit(file);
  return [...found].sort((left, right) => left - right);
}

function commentViolations(files: Source[]): string[] {
  return files.flatMap((file) => commentLines(file).map((line) => `${file.path}:${line} holds a comment`));
}

function lengthViolations(files: Source[]): string[] {
  const found: string[] = [];
  for (const source of files) {
    const lines = source.text.split("\n").length - (source.text.endsWith("\n") ? 1 : 0);
    if (lines > FILE_LINES) {
      found.push(`${source.path} has ${lines} lines, above ${FILE_LINES}`);
    }
    const file = parse(source);
    const visit = (node: ts.Node) => {
      if (ts.isFunctionLike(node) && "body" in node && node.body) {
        const start = file.getLineAndCharacterOfPosition(node.getStart(file)).line;
        const end = file.getLineAndCharacterOfPosition(node.getEnd()).line;
        if (end - start + 1 > FUNCTION_LINES) {
          found.push(`${source.path}:${start + 1} a function has ${end - start + 1} lines, above ${FUNCTION_LINES}`);
        }
      }
      ts.forEachChild(node, visit);
    };
    visit(file);
  }
  return found;
}

function untestedKit(paths: string[]): string[] {
  const present = new Set(paths);
  return paths
    .filter((path) => path.startsWith(KIT) && path.endsWith(".tsx") && !path.endsWith(".test.tsx"))
    .filter((path) => !present.has(path.replace(/\.tsx$/, ".test.tsx")))
    .map((path) => `${path} has no test beside it`);
}

describe("frontend source rules", () => {
  const files = sources();

  it("finds the application sources", () => {
    expect(files.some((file) => file.path === "app/layout.tsx")).toBe(true);
    expect(existsSync(join(SOURCE_ROOT, "shared/api/client.ts"))).toBe(true);
  });

  it("no source file holds a comment", () => {
    expect(commentViolations(files)).toEqual([]);
  });

  it("every file and function is within budget", () => {
    expect(lengthViolations(files)).toEqual([]);
  });

  it("every kit component has a test beside it", () => {
    expect(untestedKit(files.map((file) => file.path))).toEqual([]);
  });
});

describe("the guards themselves", () => {
  it("find line, block and JSX comments but not slashes in strings", () => {
    const sample: Source = {
      path: "sample.tsx",
      text: 'const a = "http://x";\n// line\nconst b = 1; /* block */\nexport const C = () => <div>{/* jsx */}</div>;\nconst r = /\\/\\//;\n',
    };
    expect(commentLines(sample)).toEqual([2, 3, 4]);
  });

  it("catch an oversized file and an oversized function", () => {
    const longFile: Source = { path: "long.ts", text: "export const a = 1;\n".repeat(FILE_LINES + 1) };
    const body = "  void 0;\n".repeat(FUNCTION_LINES - 1);
    const longFunction: Source = { path: "function.ts", text: `export function f() {\n${body}}\n` };
    const fitting: Source = { path: "fit.ts", text: `export function f() {\n${"  void 0;\n".repeat(FUNCTION_LINES - 2)}}\n` };
    expect(lengthViolations([longFile, longFunction, fitting])).toEqual([
      `long.ts has ${FILE_LINES + 1} lines, above ${FILE_LINES}`,
      `function.ts:1 a function has ${FUNCTION_LINES + 1} lines, above ${FUNCTION_LINES}`,
    ]);
  });

  it("catch a kit component without a test", () => {
    expect(untestedKit(["shared/ui/card.tsx", "shared/ui/badge.tsx", "shared/ui/badge.test.tsx", "widgets/x/ui/y.tsx"])).toEqual([
      "shared/ui/card.tsx has no test beside it",
    ]);
  });
});
