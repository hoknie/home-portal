import type { ScriptArgument } from "./schema";

export type ArgumentValues = Record<string, string | boolean>;

function text(values: ArgumentValues, name: string) {
  const value = values[name];
  return typeof value === "string" ? value : "";
}

export function toArgs(declared: ScriptArgument[], values: ArgumentValues): string[] {
  const positionals = declared.filter((argument) => !argument.option);
  const lastFilled = positionals.map((argument) => text(values, argument.name) !== "").lastIndexOf(true);
  const args = positionals.slice(0, lastFilled + 1).map((argument) => text(values, argument.name));
  for (const argument of declared.filter((entry) => entry.option)) {
    if (argument.type === "flag") {
      if (values[argument.name] === true) {
        args.push(argument.name);
      }
    } else if (text(values, argument.name) !== "") {
      args.push(argument.name, text(values, argument.name));
    }
  }
  return args;
}

export function fromArgs(declared: ScriptArgument[], args: string[]): ArgumentValues | null {
  if (declared.length === 0) {
    return null;
  }
  const options = new Map(declared.filter((argument) => argument.option).map((argument) => [argument.name, argument]));
  const values: ArgumentValues = {};
  let index = 0;
  for (const argument of declared.filter((entry) => !entry.option)) {
    if (index < args.length && !options.has(args[index])) {
      values[argument.name] = args[index];
      index += 1;
    }
  }
  while (index < args.length) {
    const option = options.get(args[index]);
    if (!option || option.name in values) {
      return null;
    }
    if (option.type === "flag") {
      values[option.name] = true;
      index += 1;
    } else if (index + 1 < args.length) {
      values[option.name] = args[index + 1];
      index += 2;
    } else {
      return null;
    }
  }
  const same = toArgs(declared, values);
  return same.length === args.length && same.every((value, position) => value === args[position]) ? values : null;
}
