"use client";

import ReactMarkdown, { type Components } from "react-markdown";

export const WEB_ADDRESS = /^https?:\/\//i;

export function safeAddress(address: string) {
  return WEB_ADDRESS.test(address.trim()) ? address : "";
}

const COMPONENTS: Components = {
  a: ({ href, children }) =>
    href ? (
      <a href={href} target="_blank" rel="noreferrer noopener" className="text-primary underline underline-offset-2">
        {children}
      </a>
    ) : (
      <span>{children}</span>
    ),
  img: () => null,
};

export function MarkdownBlock({ text }: { text: string }) {
  return (
    <div className="grid gap-2 text-sm leading-relaxed [&_code]:rounded [&_code]:bg-muted [&_code]:px-1 [&_ol]:list-decimal [&_ol]:pl-5 [&_ul]:list-disc [&_ul]:pl-5" data-block="markdown">
      <ReactMarkdown skipHtml urlTransform={safeAddress} components={COMPONENTS}>
        {text}
      </ReactMarkdown>
    </div>
  );
}
