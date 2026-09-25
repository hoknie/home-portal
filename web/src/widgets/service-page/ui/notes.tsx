import type { ReactNode } from "react";
import Markdown from "react-markdown";

function NoteLink({ href, children }: { href?: string; children?: ReactNode }) {
  return (
    <a href={href} target="_blank" rel="noreferrer" className="underline underline-offset-2">
      {children}
    </a>
  );
}

export function Notes({ text }: { text: string }) {
  return (
    <div className="grid gap-2 text-sm [&_code]:rounded [&_code]:bg-glass-tint [&_code]:px-1 [&_code]:font-mono [&_code]:text-xs [&_li]:ml-5 [&_ol]:list-decimal [&_pre]:overflow-x-auto [&_pre]:rounded-lg [&_pre]:bg-glass-tint [&_pre]:p-3 [&_ul]:list-disc [&_h1]:font-semibold [&_h2]:font-semibold [&_h3]:font-medium">
      <Markdown components={{ a: NoteLink }}>{text}</Markdown>
    </div>
  );
}
