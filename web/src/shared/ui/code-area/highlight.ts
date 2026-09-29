export const TOKEN_KINDS = ["plain", "shebang", "comment", "tag", "name", "type", "string", "variable", "keyword", "number"] as const;

export type TokenKind = (typeof TOKEN_KINDS)[number];

export type Token = { kind: TokenKind; text: string };

export const KEYWORDS = new Set([
  "if",
  "then",
  "else",
  "elif",
  "fi",
  "for",
  "in",
  "do",
  "done",
  "while",
  "until",
  "case",
  "esac",
  "function",
  "return",
  "exit",
  "set",
  "local",
  "export",
  "readonly",
  "shift",
  "echo",
  "printf",
  "exec",
  "trap",
  "source",
  "def",
  "import",
  "from",
  "class",
  "with",
  "as",
  "try",
  "except",
  "finally",
  "raise",
  "pass",
  "lambda",
  "not",
  "and",
  "or",
  "True",
  "False",
  "None",
  "const",
  "let",
  "var",
  "async",
  "await",
]);

const TAGS = ["@description", "@arg"];

function push(tokens: Token[], kind: TokenKind, text: string) {
  if (text === "") {
    return;
  }
  const last = tokens.at(-1);
  if (last && last.kind === kind) {
    last.text += text;
  } else {
    tokens.push({ kind, text });
  }
}

function comment(tokens: Token[], text: string) {
  const match = /^([#/]+\s*)(@description|@arg)(?=\s|$)(.*)$/.exec(text);
  if (!match || !TAGS.includes(match[2])) {
    push(tokens, "comment", text);
    return;
  }
  push(tokens, "comment", match[1]);
  push(tokens, "tag", match[2]);
  if (match[2] === "@description") {
    push(tokens, "comment", match[3]);
    return;
  }
  const parts = /^(\s*)(\S*)(\s*)(<[^>]*>?)?(.*)$/.exec(match[3]) ?? ["", "", "", "", "", match[3]];
  push(tokens, "comment", parts[1]);
  push(tokens, "name", parts[2]);
  push(tokens, "comment", parts[3]);
  push(tokens, "type", parts[4] ?? "");
  push(tokens, "comment", parts[5]);
}

function line(tokens: Token[], text: string, first: boolean) {
  if (first && text.startsWith("#!")) {
    push(tokens, "shebang", text);
    return;
  }
  if (text.trimStart().startsWith("//")) {
    const indent = text.length - text.trimStart().length;
    push(tokens, "plain", text.slice(0, indent));
    comment(tokens, text.slice(indent));
    return;
  }
  let index = 0;
  while (index < text.length) {
    const character = text[index];
    const rest = text.slice(index);
    if (character === "#" && (index === 0 || /\s/.test(text[index - 1]))) {
      comment(tokens, rest);
      return;
    }
    if (character === "'" || character === '"') {
      let end = index + 1;
      while (end < text.length && text[end] !== character) {
        end += character === '"' && text[end] === "\\" ? 2 : 1;
      }
      push(tokens, "string", text.slice(index, end + 1));
      index = end + 1;
      continue;
    }
    const variable = /^\$(\{[^}]*\}?|[A-Za-z_]\w*|[0-9@#?*!$-])/.exec(rest);
    if (variable) {
      push(tokens, "variable", variable[0]);
      index += variable[0].length;
      continue;
    }
    const word = /^[A-Za-z_][\w-]*/.exec(rest);
    if (word) {
      push(tokens, KEYWORDS.has(word[0]) ? "keyword" : "plain", word[0]);
      index += word[0].length;
      continue;
    }
    const number = /^\d+(\.\d+)?\b/.exec(rest);
    if (number && (index === 0 || !/\w/.test(text[index - 1]))) {
      push(tokens, "number", number[0]);
      index += number[0].length;
      continue;
    }
    push(tokens, "plain", character);
    index += 1;
  }
}

export function highlight(text: string): Token[][] {
  const lines = text.split("\n");
  return lines.map((content, index) => {
    const tokens: Token[] = [];
    line(tokens, content, index === 0);
    return tokens;
  });
}
