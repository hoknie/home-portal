import { argumentOf } from "./argument-line";
import type { ScriptArgument, ScriptHeader } from "./schema";

export const HEADER_LINES = 64;
export const HEADER_BYTES = 8 * 1024;
export const MOST_ARGUMENTS = 32;

function bounded(text: string) {
  const bytes = new TextEncoder().encode(text);
  if (bytes.length <= HEADER_BYTES) {
    return text;
  }
  return new TextDecoder().decode(bytes.slice(0, HEADER_BYTES)).replace(/�+$/, "");
}

function linesOf(text: string) {
  const lines = text.split("\n").map((line) => (line.endsWith("\r") ? line.slice(0, -1) : line));
  if (text.endsWith("\n")) {
    lines.pop();
  }
  return lines;
}

function commentOf(line: string): string | null {
  const rest = line.startsWith("//") ? line.slice(2) : line.startsWith("#") ? line.slice(1) : null;
  return rest === null ? null : rest.replace(/^[#/]+/, "");
}

function tagged(line: string, tag: string): string | null {
  if (!line.startsWith(tag)) {
    return null;
  }
  const rest = line.slice(tag.length);
  return rest === "" || /^\s/.test(rest) ? rest.trim() : null;
}

function add(header: ScriptHeader, line: number, argument: ScriptArgument) {
  if (header.arguments.length >= MOST_ARGUMENTS) {
    return problem(header, line, `declares more than ${MOST_ARGUMENTS} arguments`);
  }
  if (header.arguments.some((known) => known.name === argument.name)) {
    return problem(header, line, `declares ${argument.name} a second time`);
  }
  const optionalBefore = header.arguments.some((known) => !known.option && !known.required);
  if (!argument.option && argument.required && optionalBefore) {
    return problem(header, line, `declares the required ${argument.name} after an optional positional argument`);
  }
  header.arguments.push(argument);
}

function problem(header: ScriptHeader, line: number, message: string) {
  header.problems.push({ line, message: `line ${line} ${message}` });
}

function read(header: ScriptHeader, line: number, comment: string) {
  const description = tagged(comment, "@description");
  if (description !== null) {
    if (header.description !== null) {
      problem(header, line, "is a second @description; only the first one counts");
    } else if (description === "") {
      problem(header, line, "is an @description without text");
    } else {
      header.description = description;
    }
    return;
  }
  const declaration = tagged(comment, "@arg");
  if (declaration === null) {
    return;
  }
  try {
    add(header, line, argumentOf(declaration));
  } catch (error) {
    problem(header, line, `declares an argument that ${(error as Error).message}`);
  }
}

export function parseHeader(text: string): ScriptHeader {
  const header: ScriptHeader = {
    description: null,
    arguments: [],
    problems: [],
  };
  const lines = linesOf(bounded(text)).slice(0, HEADER_LINES);
  for (const [index, line] of lines.entries()) {
    const number = index + 1;
    if (number === 1 && line.startsWith("#!")) {
      continue;
    }
    const trimmed = line.trimStart();
    if (trimmed === "") {
      continue;
    }
    const comment = commentOf(trimmed);
    if (comment === null) {
      break;
    }
    read(header, number, comment.trim());
  }
  return header;
}
