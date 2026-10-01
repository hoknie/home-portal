const DEFINITIONS = "#/$defs/";

export class UnsupportedSchema extends Error {
  constructor(path, reason) {
    super(`${path}: ${reason}`);
    this.name = "UnsupportedSchema";
  }
}

export function camel(name) {
  const words = name.split(/[^A-Za-z0-9]+/).filter(Boolean);
  return words.map((word, index) => (index === 0 ? word[0].toLowerCase() + word.slice(1) : word[0].toUpperCase() + word.slice(1))).join("");
}

export function pascal(name) {
  const value = camel(name);
  return value[0].toUpperCase() + value.slice(1);
}

function literal(value) {
  return JSON.stringify(value);
}

function types(schema) {
  return Array.isArray(schema.type) ? schema.type : schema.type === undefined ? [] : [schema.type];
}

function withoutNull(schema) {
  const rest = types(schema).filter((type) => type !== "null");
  return { ...schema, type: rest.length === 1 ? rest[0] : rest };
}

function nullVariant(schema) {
  return types(schema).length === 1 && types(schema)[0] === "null";
}

function refName(reference, path) {
  if (!reference.startsWith(DEFINITIONS)) {
    throw new UnsupportedSchema(path, `a reference outside ${DEFINITIONS}: ${reference}`);
  }
  return reference.slice(DEFINITIONS.length);
}

function stringEnum(values, schema, path) {
  if (!values.every((value) => typeof value === "string")) {
    throw new UnsupportedSchema(path, "an enum of values other than text");
  }
  const open = schema["x-open"];
  if (open !== undefined && !values.includes(open)) {
    throw new UnsupportedSchema(path, `x-open names ${open}, which is not one of the values`);
  }
  const base = `z.enum([${values.map(literal).join(", ")}])`;
  return open === undefined ? base : `${base}.catch(${literal(open)})`;
}

function constants(variants) {
  const values = variants.map((variant) => variant.const ?? (variant.enum?.length === 1 ? variant.enum[0] : undefined));
  return values.every((value) => typeof value === "string") ? values : null;
}

function objectOf(schema, path, context) {
  const properties = schema.properties ?? {};
  const required = new Set(schema.required ?? []);
  const entries = Object.entries(properties).map(([key, property]) => {
    const inner = expression(property, `${path}.${key}`, context);
    return `${literal(key)}: ${required.has(key) ? inner : optional(inner, property)}`;
  });
  const extra = schema.additionalProperties;
  if (entries.length === 0 && extra && extra !== true && typeof extra === "object") {
    return `z.record(z.string(), ${expression(extra, `${path}.*`, context)})`;
  }
  if (entries.length === 0 && extra === true) {
    return "z.record(z.string(), z.unknown())";
  }
  return `z.object({ ${entries.join(", ")} })`;
}

function optional(inner, property) {
  if (property.default !== undefined) {
    return `${inner}.default(${literal(property.default)})`;
  }
  if (inner.endsWith(".nullable()")) {
    return `${inner}.default(null)`;
  }
  if (inner.startsWith("z.array(")) {
    return `${inner}.default([])`;
  }
  if (inner === "z.boolean()") {
    return `${inner}.default(false)`;
  }
  if (inner === "z.number()") {
    return `${inner}.default(0)`;
  }
  return `${inner}.optional()`;
}

function merged(parts, path) {
  const objects = parts.map((part) => withoutNull(part));
  if (!objects.every((part) => types(part)[0] === "object" || part.properties)) {
    throw new UnsupportedSchema(path, "allOf of something other than objects");
  }
  return {
    type: "object",
    properties: Object.assign({}, ...objects.map((part) => part.properties ?? {})),
    required: objects.flatMap((part) => part.required ?? []),
  };
}

export function expression(schema, path, context) {
  if (schema === true || (typeof schema === "object" && Object.keys(schema).filter((key) => !["description", "title", "default"].includes(key)).length === 0)) {
    return "z.unknown()";
  }
  if (schema === false) {
    throw new UnsupportedSchema(path, "a schema that accepts nothing");
  }
  if (schema.$ref) {
    const name = refName(schema.$ref, path);
    context.used.add(name);
    return `${camel(name)}Schema`;
  }
  const variants = schema.anyOf ?? schema.oneOf;
  if (variants) {
    const real = variants.filter((variant) => !nullVariant(variant));
    const nullable = real.length < variants.length;
    const values = constants(real);
    const inner = values !== null ? stringEnum(values, schema, path) : real.length === 1 ? expression(real[0], path, context) : `z.union([${real.map((variant, index) => expression(variant, `${path}|${index}`, context)).join(", ")}])`;
    return nullable ? `${inner}.nullable()` : inner;
  }
  if (schema.allOf) {
    return expression(merged(schema.allOf, path), path, context);
  }
  if (schema.const !== undefined) {
    return `z.literal(${literal(schema.const)})`;
  }
  const kinds = types(schema);
  const nullable = kinds.includes("null");
  const rest = kinds.filter((kind) => kind !== "null");
  if (rest.length > 1) {
    throw new UnsupportedSchema(path, `a value of several types (${rest.join(", ")})`);
  }
  const inner = valueOf(rest[0], schema, path, context);
  return nullable ? `${inner}.nullable()` : inner;
}

function valueOf(kind, schema, path, context) {
  if (schema.enum) {
    return stringEnum(schema.enum.filter((value) => value !== null), schema, path);
  }
  switch (kind) {
    case "string":
      return "z.string()";
    case "integer":
    case "number":
      return "z.number()";
    case "boolean":
      return "z.boolean()";
    case "array":
      if (schema.items === undefined || Array.isArray(schema.items) || schema.prefixItems) {
        throw new UnsupportedSchema(path, "an array without one item schema");
      }
      return `z.array(${expression(schema.items, `${path}[]`, context)})`;
    case "object":
      return objectOf(schema, path, context);
    case undefined:
      if (schema.properties) {
        return objectOf(schema, path, context);
      }
      throw new UnsupportedSchema(path, "a schema without a type");
    default:
      throw new UnsupportedSchema(path, `the type ${kind}`);
  }
}

function ordered(definitions, root) {
  const done = [];
  const visiting = new Set();
  const visit = (name, path) => {
    if (done.includes(name)) {
      return;
    }
    if (visiting.has(name)) {
      throw new UnsupportedSchema(path, `a recursive definition ${name}`);
    }
    visiting.add(name);
    for (const next of definitions.get(name).used) {
      visit(next, `${path} → ${next}`);
    }
    visiting.delete(name);
    done.push(name);
  };
  for (const name of root.used) {
    visit(name, name);
  }
  return done;
}

export function generate(schema, answer) {
  const name = schema.title ?? pascal(answer);
  const definitions = new Map();
  for (const [key, definition] of Object.entries(schema.$defs ?? {})) {
    const context = { used: new Set() };
    definitions.set(key, { context, code: expression(definition, `${answer}#${key}`, context), used: context.used });
  }
  const root = { used: new Set() };
  const code = expression({ ...schema, $defs: undefined, title: undefined, $schema: undefined }, answer, root);
  const lines = ['import { z } from "zod";', ""];
  for (const key of ordered(definitions, root)) {
    lines.push(`export const ${camel(key)}Schema = ${definitions.get(key).code};`, "", `export type ${pascal(key)} = z.infer<typeof ${camel(key)}Schema>;`, "");
  }
  if (!definitions.has(name)) {
    lines.push(`export const ${camel(name)}Schema = ${code};`, "", `export type ${pascal(name)} = z.infer<typeof ${camel(name)}Schema>;`, "");
  }
  lines.push(`export const schema = ${camel(name)}Schema;`, "");
  return lines.join("\n");
}
