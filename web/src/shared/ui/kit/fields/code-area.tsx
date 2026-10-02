"use client";

import { cn } from "cn";
import { type KeyboardEvent, useMemo, useRef } from "react";

import { type TokenKind, highlight } from "./highlight";

export const INDENT = "  ";

export const TOKEN_CLASSES: Record<TokenKind, string> = {
  plain: "",
  shebang: "text-muted-foreground",
  comment: "text-muted-foreground italic",
  tag: "font-semibold not-italic text-palette-violet",
  name: "not-italic text-palette-blue",
  type: "not-italic text-palette-amber",
  string: "text-palette-green",
  variable: "text-palette-blue",
  keyword: "font-medium text-palette-violet",
  number: "text-palette-amber",
};

export type CodeAreaProps = {
  id?: string;
  value: string;
  onChange: (value: string) => void;
  onSave?: () => void;
  readOnly?: boolean;
  className?: string;
  "aria-label"?: string;
  "aria-describedby"?: string;
};

const TEXT = "m-0 px-3 py-2 font-mono text-sm leading-5 whitespace-pre [tab-size:2]";

export function CodeArea({ id, value, onChange, onSave, readOnly = false, className, ...aria }: CodeAreaProps) {
  const gutter = useRef<HTMLDivElement>(null);
  const painted = useRef<HTMLPreElement>(null);
  const lines = useMemo(() => highlight(value), [value]);
  const pressed = (event: KeyboardEvent<HTMLTextAreaElement>) => {
    if ((event.metaKey || event.ctrlKey) && event.key.toLowerCase() === "s") {
      event.preventDefault();
      onSave?.();
      return;
    }
    if (event.key !== "Tab" || event.shiftKey || event.altKey || event.metaKey || event.ctrlKey || readOnly) {
      return;
    }
    event.preventDefault();
    const field = event.currentTarget;
    const start = field.selectionStart;
    const end = field.selectionEnd;
    onChange(`${value.slice(0, start)}${INDENT}${value.slice(end)}`);
    requestAnimationFrame(() => {
      field.selectionStart = start + INDENT.length;
      field.selectionEnd = start + INDENT.length;
    });
  };
  return (
    <div
      className={cn(
        "flex min-h-64 overflow-hidden surface-inset rounded-lg focus-within:border-ring focus-within:ring-[3px] focus-within:ring-ring/50",
        className,
      )}
    >
      <div
        ref={gutter}
        aria-hidden
        className="shrink-0 overflow-hidden border-r border-glass-edge bg-glass-tint py-2 pr-2 pl-3 text-right font-mono text-sm leading-5 text-muted-foreground select-none"
      >
        {lines.map((_, index) => (
          <div key={index}>{index + 1}</div>
        ))}
      </div>
      <div className="relative min-w-0 flex-1">
        <pre ref={painted} aria-hidden data-testid="code-highlight" className={cn(TEXT, "pointer-events-none absolute inset-0 overflow-hidden")}>
          {lines.map((tokens, index) => (
            <div key={index}>
              {tokens.length === 0
                ? " "
                : tokens.map((token, position) => (
                    <span key={position} data-token={token.kind} className={TOKEN_CLASSES[token.kind]}>
                      {token.text}
                    </span>
                  ))}
            </div>
          ))}
        </pre>
        <textarea
          id={id}
          {...aria}
          value={value}
          readOnly={readOnly}
          spellCheck={false}
          autoCapitalize="off"
          autoCorrect="off"
          wrap="off"
          onChange={(change) => onChange(change.target.value)}
          onKeyDown={pressed}
          onScroll={(scroll) => {
            const { scrollTop, scrollLeft } = scroll.currentTarget;
            if (gutter.current) {
              gutter.current.scrollTop = scrollTop;
            }
            if (painted.current) {
              painted.current.scrollTop = scrollTop;
              painted.current.scrollLeft = scrollLeft;
            }
          }}
          className={cn(
            TEXT,
            "relative block size-full min-h-full resize-none overflow-auto bg-transparent text-transparent caret-foreground outline-none selection:bg-primary/25 selection:text-transparent",
          )}
        />
      </div>
    </div>
  );
}
