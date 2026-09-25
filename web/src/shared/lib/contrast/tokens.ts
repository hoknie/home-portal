import { type Color, parseOklch } from "./color";

export type Tokens = Map<string, string>;

const DECLARATION = /(--[\w-]+)\s*:\s*([^;]+);/g;
const REFERENCE = /^var\(\s*(--[\w-]+)\s*\)$/;

export function blockAfter(css: string, header: string): string {
  const start = css.indexOf(header);
  if (start < 0) {
    throw new Error(`no block starts with ${header}`);
  }
  const open = css.indexOf("{", start + header.length - 1);
  let depth = 0;
  for (let index = open; index < css.length; index += 1) {
    if (css[index] === "{") {
      depth += 1;
    } else if (css[index] === "}") {
      depth -= 1;
      if (depth === 0) {
        return css.slice(open + 1, index);
      }
    }
  }
  throw new Error(`the block ${header} is not closed`);
}

export function declarations(block: string): Tokens {
  return new Map([...block.matchAll(DECLARATION)].map((match) => [match[1], match[2].trim()]));
}

export function themeTokens(css: string, selector: ":root" | ".dark"): Tokens {
  return declarations(blockAfter(css, `\n${selector} {`));
}

export function tokenValue(tokens: Tokens, name: string): string {
  const value = tokens.get(name);
  if (value === undefined) {
    throw new Error(`missing token ${name}`);
  }
  const reference = REFERENCE.exec(value);
  return reference ? tokenValue(tokens, reference[1]) : value;
}

export function tokenColor(tokens: Tokens, name: string): Color {
  return parseOklch(tokenValue(tokens, name));
}
