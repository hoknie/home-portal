import Link from "next/link";

import { cn } from "@/shared/lib/cn";

export type Reference = { kind: string; text: string; href: string | null; mono?: boolean };

export function ItemReference({ kind, text, href, mono = false, also }: Reference & { also?: Reference | null }) {
  const shown = (reference: Reference) => {
    const className = cn("truncate", reference.mono ? "font-mono text-xs" : "text-sm");
    return reference.href ? (
      <Link href={reference.href} className={cn(className, "block hover:underline")}>
        {reference.text}
      </Link>
    ) : (
      <span className={cn(className, "block text-muted-foreground")}>{reference.text}</span>
    );
  };
  return (
    <span className="grid min-w-0 gap-0.5">
      <span className="text-xs text-muted-foreground">{kind}</span>
      {shown({ kind, text, href, mono })}
      {also ? (
        <span className="grid min-w-0 gap-0.5 pt-0.5">
          <span className="text-xs text-muted-foreground">{also.kind}</span>
          {shown(also)}
        </span>
      ) : null}
    </span>
  );
}
