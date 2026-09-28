export type TemplateSuggestion = {
  value: string;
  label?: string;
  description?: string;
  example?: string;
  group?: string;
  warning?: string;
  disabled?: boolean;
};

export type TemplateRange = { start: number; end: number; message: string; severity?: "error" | "warning" };

export type Trigger = "braces" | "always";

export type Mode = Trigger | "filters" | "arguments";

export type Completion = { start: number; end: number; query: string; subject?: string; chain?: string };

export type Segment = { text: string; start: number; template: boolean; problem: TemplateRange | null };

export const MOST_SHOWN = 60;
export const OPEN = "{{";
export const CLOSE = "}}";

export function completionAt(value: string, caret: number, trigger: Trigger): Completion | null {
  if (trigger === "always") {
    return { start: 0, end: value.length, query: value };
  }
  const open = value.lastIndexOf(OPEN, caret - OPEN.length);
  if (open < 0) {
    return null;
  }
  const between = value.slice(open + OPEN.length, caret);
  if (between.includes(CLOSE) || /\s/.test(between)) {
    return null;
  }
  return { start: open + OPEN.length, end: caret, query: between };
}

export const BAR = "|";

export function filterCompletionAt(value: string, caret: number): Completion | null {
  const open = value.lastIndexOf(OPEN, caret - OPEN.length);
  if (open < 0) {
    return null;
  }
  const between = value.slice(open + OPEN.length, caret);
  const first = between.indexOf(BAR);
  const last = between.lastIndexOf(BAR);
  if (between.includes(CLOSE) || first < 0) {
    return null;
  }
  const typed = between.slice(last + 1);
  const query = typed.trimStart();
  if (!/^[a-z0-9_]*$/.test(query)) {
    return null;
  }
  return {
    start: caret - query.length,
    end: caret,
    query,
    subject: between.slice(0, first).trim(),
    chain: first === last ? "" : between.slice(first + 1, last).trim(),
  };
}

export function argumentCompletionAt(value: string, caret: number): Completion | null {
  const open = value.lastIndexOf(OPEN, caret - OPEN.length);
  if (open < 0) {
    return null;
  }
  const between = value.slice(open + OPEN.length, caret);
  const bar = between.lastIndexOf(BAR);
  if (between.includes(CLOSE) || bar < 0) {
    return null;
  }
  const call = between.slice(bar + 1);
  const paren = call.indexOf("(");
  if (paren < 0 || call.includes(")") || (call.split('"').length - 1) % 2 === 1 || (call.split("'").length - 1) % 2 === 1) {
    return null;
  }
  const typed = /[(,]\s*([A-Za-z0-9_.-]*)$/.exec(call);
  if (typed === null) {
    return null;
  }
  return { start: caret - typed[1].length, end: caret, query: typed[1] };
}

export function matching(suggestions: TemplateSuggestion[], query: string): TemplateSuggestion[] {
  const needle = query.trim().toLowerCase();
  if (needle === "") {
    return suggestions.slice(0, MOST_SHOWN);
  }
  const text = (suggestion: TemplateSuggestion) => `${suggestion.value} ${suggestion.label ?? ""}`.toLowerCase();
  const starts = suggestions.filter((suggestion) => suggestion.value.toLowerCase().startsWith(needle) || (suggestion.label ?? "").toLowerCase().startsWith(needle));
  const contains = suggestions.filter((suggestion) => !starts.includes(suggestion) && text(suggestion).includes(needle));
  return [...starts, ...contains].slice(0, MOST_SHOWN);
}

export function applied(value: string, completion: Completion, suggestion: TemplateSuggestion, trigger: Mode): { value: string; caret: number } {
  if (trigger === "always") {
    return { value: suggestion.value, caret: suggestion.value.length };
  }
  if (trigger === "arguments") {
    const next = `${value.slice(0, completion.start)}${suggestion.value}${value.slice(completion.end)}`;
    return { value: next, caret: completion.start + suggestion.value.length };
  }
  if (trigger === "filters") {
    const before = value.slice(0, completion.start);
    const spaced = before.endsWith(BAR) ? `${before} ` : before;
    const after = value.slice(completion.end);
    const next = `${spaced}${suggestion.value}${after}`;
    return { value: next, caret: spaced.length + suggestion.value.length };
  }
  const after = value.slice(completion.end);
  const closing = after.startsWith(CLOSE) ? "" : CLOSE;
  const next = `${value.slice(0, completion.start)}${suggestion.value}${closing}${after}`;
  return { value: next, caret: completion.start + suggestion.value.length + CLOSE.length };
}

export function templateRanges(value: string): { start: number; end: number }[] {
  const ranges: { start: number; end: number }[] = [];
  let from = 0;
  for (;;) {
    const open = value.indexOf(OPEN, from);
    const close = open < 0 ? -1 : value.indexOf(CLOSE, open + OPEN.length);
    if (open < 0 || close < 0) {
      return ranges;
    }
    ranges.push({ start: open, end: close + CLOSE.length });
    from = close + CLOSE.length;
  }
}

export function segmentsOf(value: string, problems: TemplateRange[]): Segment[] {
  const templates = templateRanges(value);
  const cuts = new Set([0, value.length]);
  for (const range of [...templates, ...problems]) {
    cuts.add(Math.max(0, Math.min(range.start, value.length)));
    cuts.add(Math.max(0, Math.min(range.end, value.length)));
  }
  const points = [...cuts].sort((left, right) => left - right);
  const segments: Segment[] = [];
  for (let index = 0; index < points.length - 1; index += 1) {
    const start = points[index];
    const end = points[index + 1];
    if (end > start) {
      segments.push({
        text: value.slice(start, end),
        start,
        template: templates.some((range) => range.start <= start && end <= range.end),
        problem: problems.find((range) => range.start <= start && end <= range.end) ?? null,
      });
    }
  }
  return segments;
}
