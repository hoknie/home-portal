import type { ScriptArgument } from "./schema";

type Kind = Pick<ScriptArgument, "type" | "choices">;

const NUMBER = /^[+-]?(\d+\.?\d*|\.\d+)([eE][+-]?\d+)?$/;

function accepts(kind: Kind, value: string) {
  switch (kind.type) {
    case "text":
      return true;
    case "number":
      return NUMBER.test(value) && Number.isFinite(Number(value));
    case "flag":
      return false;
    default:
      return kind.choices.includes(value);
  }
}

function describeKind(kind: Kind) {
  return kind.type === "choice" ? `choice of ${kind.choices.join(", ")}` : kind.type;
}

function nameOf(written: string): {
  name: string;
  option: boolean;
  optional: boolean;
} {
  const option = written.startsWith("--");
  const optional = !option && written.endsWith("?");
  const base = option ? written.slice(2) : optional ? written.slice(0, -1) : written;
  const valid = /^[a-z]/.test(base) && /^[a-z0-9_-]*$/.test(base);
  if (!valid) {
    throw new Error(`names ${written}, but a name is lowercase letters, digits, _ and -, starting with a letter, or -- before such a name`);
  }
  return { name: option ? written : base, option, optional };
}

function kindOf(text: string): Kind {
  if (text === "text" || text === "number" || text === "flag") {
    return { type: text, choices: [] };
  }
  if (text.includes("|")) {
    const choices = text.split("|").map((choice) => choice.trim());
    if (choices.some((choice) => choice === "")) {
      throw new Error("has a choice with an empty value");
    }
    return { type: "choice", choices };
  }
  throw new Error(`has the type ${text}, but ${text} is not a known type; use text, number, flag or choices such as a|b`);
}

function splitFirst(text: string, separator: RegExp | string): [string, string] | null {
  const index = typeof separator === "string" ? text.indexOf(separator) : text.search(separator);
  if (index < 0) {
    return null;
  }
  const width = typeof separator === "string" ? separator.length : 1;
  return [text.slice(0, index), text.slice(index + width)];
}

export function argumentOf(line: string): ScriptArgument {
  const trimmed = line.trim();
  const [written, rest] = splitFirst(trimmed, /\s/) ?? [trimmed, ""];
  const { name, option, optional } = nameOf(written);
  const after = rest.trimStart();
  let kind: Kind;
  let defaultValue: string | null = null;
  let description: string;
  if (after.startsWith("<")) {
    const closed = splitFirst(after.slice(1), ">");
    if (!closed) {
      throw new Error("has a type that is not closed with >");
    }
    const [spec, tail] = closed;
    const withDefault = splitFirst(spec, "=");
    kind = kindOf(withDefault ? withDefault[0].trim() : spec.trim());
    defaultValue = withDefault ? withDefault[1].trim() : null;
    description = tail.trim();
  } else {
    kind = { type: option ? "flag" : "text", choices: [] };
    description = after;
  }
  if (kind.type === "flag" && !option) {
    throw new Error(`${name} is a flag, which must be an option such as --${name}`);
  }
  if (kind.type === "flag" && defaultValue !== null) {
    throw new Error(`${name} is a flag, and a flag takes no default`);
  }
  if (defaultValue !== null && !accepts(kind, defaultValue)) {
    throw new Error(`${name} has the default ${defaultValue}, which is not a ${describeKind(kind)}`);
  }
  return {
    name,
    option,
    required: !option && !optional && defaultValue === null,
    type: kind.type,
    choices: kind.choices,
    default: defaultValue,
    description,
  };
}
