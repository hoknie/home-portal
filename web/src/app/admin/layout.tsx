import type { ReactNode } from "react";

import { AppShell, AreaGate } from "@/widgets/app-shell";

export default function ShellLayout({ children }: { children: ReactNode }) {
  return (
    <AppShell>
      <AreaGate>{children}</AreaGate>
    </AppShell>
  );
}
