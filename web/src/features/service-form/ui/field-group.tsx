import type { ReactNode } from "react";

export type FieldGroupProps = {
  id: string;
  title: string;
  children: ReactNode;
};

export function FieldGroup({ id, title, children }: FieldGroupProps) {
  const heading = `${id}-title`;
  return (
    <div role="group" aria-labelledby={heading} className="grid gap-4">
      <h3 id={heading} className="text-sm font-medium text-muted-foreground">
        {title}
      </h3>
      {children}
    </div>
  );
}
