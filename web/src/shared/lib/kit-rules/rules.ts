import ts from "typescript";

export const KIT = "shared/ui/kit/";

export const RAW_ELEMENTS = ["button", "input", "select", "textarea", "table", "thead", "tbody", "tr", "th", "td", "h1", "h2", "h3", "h4", "h5", "h6"];

export const RADIX = /^(radix-ui|@radix-ui\/.+)$/;

const COLOURS = "slate|gray|zinc|neutral|stone|red|orange|amber|yellow|lime|green|emerald|teal|cyan|sky|blue|indigo|violet|purple|fuchsia|pink|rose";

const START = String.raw`(?<![\w-])(?:[\w-]+:)*`;

export const CLASS_RULES: { name: string; pattern: RegExp }[] = [
  { name: "an arbitrary value", pattern: new RegExp(String.raw`${START}((?:text|bg|shadow|rounded(?:-[a-z]+)?|from|via|to)-\[[^\]\s]+\])`, "g") },
  { name: "a palette colour", pattern: new RegExp(String.raw`${START}((?:text|bg|border|ring|outline|fill|stroke|from|via|to|decoration|divide|shadow)-(?:white|black|(?:${COLOURS})-\d{2,3})(?:\/\d+)?)(?![\w-])`, "g") },
  { name: "a glass, surface, shadow or blur class", pattern: new RegExp(String.raw`${START}(glass-[\w-]+|surface-[\w-]+|shadow(?:-(?:2xs|xs|sm|md|lg|xl|2xl|inner|none))?|backdrop-[\w-]+|blur(?:-[\w-]+)?)(?![\w/.[-])`, "g") },
  { name: "a text tone with opacity", pattern: new RegExp(String.raw`${START}(text-[a-z-]+\/\d+)(?![\w-])`, "g") },
  { name: "a text size off the scale", pattern: new RegExp(String.raw`${START}(text-(?:xl|[3-9]xl))(?![\w-])`, "g") },
];

export type Source = { path: string; text: string };

function parse(source: Source) {
  const kind = source.path.endsWith(".tsx") ? ts.ScriptKind.TSX : ts.ScriptKind.TS;
  return ts.createSourceFile(source.path, source.text, ts.ScriptTarget.Latest, true, kind);
}

export function governed(path: string): boolean {
  return !path.startsWith(KIT) && /\.tsx?$/.test(path) && !/\.test\.tsx?$/.test(path) && !path.startsWith("shared/api/generated/") && !path.startsWith("shared/lib/kit-rules/");
}

export function violations(source: Source): string[] {
  const file = parse(source);
  const found: string[] = [];
  const at = (node: ts.Node) => `${source.path}:${file.getLineAndCharacterOfPosition(node.getStart(file)).line + 1}`;
  const visit = (node: ts.Node) => {
    if ((ts.isJsxOpeningElement(node) || ts.isJsxSelfClosingElement(node)) && RAW_ELEMENTS.includes(node.tagName.getText(file))) {
      found.push(`${at(node)} renders a raw <${node.tagName.getText(file)}>`);
    }
    if (ts.isImportDeclaration(node) && ts.isStringLiteral(node.moduleSpecifier) && RADIX.test(node.moduleSpecifier.text)) {
      found.push(`${at(node)} imports ${node.moduleSpecifier.text}`);
    }
    if (ts.isStringLiteral(node) || ts.isNoSubstitutionTemplateLiteral(node) || ts.isTemplateHead(node) || ts.isTemplateMiddle(node) || ts.isTemplateTail(node)) {
      for (const rule of CLASS_RULES) {
        for (const match of node.text.matchAll(rule.pattern)) {
          found.push(`${at(node)} uses ${rule.name}: ${match[1]}`);
        }
      }
    }
    ts.forEachChild(node, visit);
  };
  visit(file);
  return found;
}
