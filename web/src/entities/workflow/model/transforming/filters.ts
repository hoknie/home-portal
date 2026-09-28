import type { FilterCall } from "./parse";
import { type ValueType, atKey, canonical, described, isObject, orderOf, sameValue, textOf, typeOfValue } from "./values";

type Argument = { name: string; type: "text" | "number" | "any"; required: boolean };

type Filter = { accepts: ValueType[]; gives: ValueType; element: boolean; arguments: Argument[] };

const text = (name: string, required = true): Argument => ({ name, type: "text", required });
const number = (name: string, required = true): Argument => ({ name, type: "number", required });
const any = (name: string): Argument => ({ name, type: "any", required: true });

export const FILTERS: Record<string, Filter> = {
  upper: { accepts: ["text"], gives: "text", element: true, arguments: [] },
  lower: { accepts: ["text"], gives: "text", element: true, arguments: [] },
  trim: { accepts: ["text"], gives: "text", element: true, arguments: [] },
  replace: { accepts: ["text"], gives: "text", element: true, arguments: [text("from"), text("to")] },
  split: { accepts: ["text"], gives: "list", element: false, arguments: [text("separator")] },
  slice: { accepts: ["text", "list"], gives: "any", element: false, arguments: [number("start"), number("end", false)] },
  starts_with: { accepts: ["text"], gives: "boolean", element: true, arguments: [text("text")] },
  contains: { accepts: ["text", "list"], gives: "boolean", element: false, arguments: [any("value")] },
  length: { accepts: ["text", "list", "object"], gives: "number", element: false, arguments: [] },
  default: { accepts: ["any"], gives: "any", element: false, arguments: [any("value")] },
  number: { accepts: ["text", "number"], gives: "number", element: true, arguments: [] },
  round: { accepts: ["number"], gives: "number", element: true, arguments: [number("digits", false)] },
  floor: { accepts: ["number"], gives: "number", element: true, arguments: [] },
  ceil: { accepts: ["number"], gives: "number", element: true, arguments: [] },
  abs: { accepts: ["number"], gives: "number", element: true, arguments: [] },
  first: { accepts: ["list"], gives: "any", element: false, arguments: [] },
  last: { accepts: ["list"], gives: "any", element: false, arguments: [] },
  join: { accepts: ["list"], gives: "text", element: false, arguments: [text("separator")] },
  sort: { accepts: ["list"], gives: "list", element: false, arguments: [] },
  reverse: { accepts: ["list"], gives: "list", element: false, arguments: [] },
  unique: { accepts: ["list"], gives: "list", element: false, arguments: [] },
  pluck: { accepts: ["list"], gives: "list", element: false, arguments: [text("key")] },
  sum: { accepts: ["list"], gives: "number", element: false, arguments: [] },
  min: { accepts: ["list"], gives: "number", element: false, arguments: [] },
  max: { accepts: ["list"], gives: "number", element: false, arguments: [] },
  keys: { accepts: ["object"], gives: "list", element: false, arguments: [] },
  values: { accepts: ["object"], gives: "list", element: false, arguments: [] },
  get: { accepts: ["object", "list"], gives: "any", element: false, arguments: [text("key")] },
  json: { accepts: ["any"], gives: "text", element: false, arguments: [] },
  parse: { accepts: ["text"], gives: "any", element: false, arguments: [] },
};

const NUMBER_TEXT = /^[+-]?(\d+\.?\d*|\.\d+)([eE][+-]?\d+)?$/;

export function accepted(types: ValueType[]) {
  return types.map(described).join(" or ");
}

export function takes(types: ValueType[], type: ValueType) {
  return types.includes("any") || types.includes(type);
}

function awayFromZero(number: number) {
  return Math.sign(number) * Math.round(Math.abs(number));
}

function bounds(length: number, start: unknown, end: unknown): [number, number] {
  const place = (value: unknown, fallback: number) => {
    if (typeof value !== "number" || !Number.isInteger(value)) {
      return fallback;
    }
    return value < 0 ? Math.max(0, length + value) : Math.min(value, length);
  };
  const from = place(start, 0);
  const to = place(end, length);
  return [Math.min(from, to), to];
}

function aggregate(name: string, items: unknown[]): unknown {
  const numbers: number[] = [];
  for (const [index, item] of items.entries()) {
    if (typeof item !== "number") {
      throw new Error(`${name} takes a list of numbers and got ${described(typeOfValue(item))} at position ${index}`);
    }
    numbers.push(item);
  }
  if (name === "sum") {
    return numbers.reduce((total, number) => total + number, 0);
  }
  if (numbers.length === 0) {
    return null;
  }
  return name === "min" ? Math.min(...numbers) : Math.max(...numbers);
}

function sortedKeys(value: Record<string, unknown>) {
  return Object.keys(value).sort();
}

function applyTyped(name: string, value: unknown, argument: (index: number) => unknown): unknown {
  const text = (index: number) => textOf(argument(index));
  if (typeof value === "string") {
    const characters = [...value];
    switch (name) {
      case "upper":
        return value.toUpperCase();
      case "lower":
        return value.toLowerCase();
      case "trim":
        return value.trim();
      case "replace":
        return value.split(text(0)).join(text(1));
      case "split":
        return value.split(text(0));
      case "slice": {
        const [from, to] = bounds(characters.length, argument(0), argument(1));
        return characters.slice(from, to).join("");
      }
      case "starts_with":
        return value.startsWith(text(0));
      case "contains":
        return value.includes(text(0));
      case "length":
        return characters.length;
      case "number":
        if (!NUMBER_TEXT.test(value.trim())) {
          throw new Error(`number cannot read "${value}" as a number`);
        }
        return Number(value.trim());
      case "parse":
        try {
          return JSON.parse(value) as unknown;
        } catch {
          throw new Error("parse cannot read the text as JSON");
        }
    }
  }
  if (typeof value === "number") {
    switch (name) {
      case "number":
        return value;
      case "round": {
        const digits = argument(0);
        const scale = 10 ** (typeof digits === "number" && Number.isInteger(digits) ? Math.min(Math.max(digits, 0), 10) : 0);
        return awayFromZero(value * scale) / scale;
      }
      case "floor":
        return Math.floor(value);
      case "ceil":
        return Math.ceil(value);
      case "abs":
        return Math.abs(value);
    }
  }
  if (Array.isArray(value)) {
    switch (name) {
      case "slice": {
        const [from, to] = bounds(value.length, argument(0), argument(1));
        return value.slice(from, to);
      }
      case "contains":
        return value.some((item) => sameValue(item, argument(0)) || textOf(item) === textOf(argument(0)));
      case "length":
        return value.length;
      case "first":
        return value[0] ?? null;
      case "last":
        return value.at(-1) ?? null;
      case "join":
        return value.map(textOf).join(text(0));
      case "sort":
        return [...value].sort(orderOf);
      case "reverse":
        return [...value].reverse();
      case "unique":
        return value.filter((item, index) => value.findIndex((other) => sameValue(other, item)) === index);
      case "pluck":
        return value.map((item) => atKey(item, text(0)));
      case "sum":
      case "min":
      case "max":
        return aggregate(name, value);
      case "get":
        return atKey(value, text(0));
    }
  }
  if (isObject(value)) {
    switch (name) {
      case "length":
        return Object.keys(value).length;
      case "keys":
        return sortedKeys(value);
      case "values":
        return sortedKeys(value).map((key) => value[key]);
      case "get":
        return atKey(value, text(0));
    }
  }
  throw new Error(`${name} is not a filter`);
}

export function applyFilter(value: unknown, call: FilterCall): unknown {
  const accepts = FILTERS[call.name]?.accepts;
  if (accepts === undefined) {
    throw new Error(`${call.name} is not a filter`);
  }
  const current = value === undefined ? null : value;
  if (current === null && call.name !== "default" && call.name !== "json") {
    return null;
  }
  const got = typeOfValue(current);
  if (FILTERS[call.name].element && Array.isArray(current) && !takes(accepts, got)) {
    return current.map((item, index) => {
      const kind = typeOfValue(item);
      if (item !== null && item !== undefined && !takes(accepts, kind)) {
        throw new Error(`${call.name} takes ${accepted(accepts)} and got ${described(kind)} at position ${index}`);
      }
      return applyFilter(item, call);
    });
  }
  if (!takes(accepts, got)) {
    throw new Error(`${call.name} takes ${accepted(accepts)} and got ${described(got)}`);
  }
  const argument = (index: number) => call.arguments[index] ?? null;
  if (call.name === "default") {
    return current === null || current === "" ? argument(0) : current;
  }
  if (call.name === "json") {
    return canonical(current);
  }
  return applyTyped(call.name, current, argument);
}

export function applyChain(value: unknown, filters: FilterCall[]): unknown {
  return filters.reduce<unknown>((current, call) => applyFilter(current, call), value);
}
