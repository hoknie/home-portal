import type { ReactNode } from "react";

import { SiteFrame } from "@/widgets/app-shell";
import { SiteHeader } from "@/widgets/site-header";

export default function SiteLayout({ children }: { children: ReactNode }) {
  return <SiteFrame guest={<SiteHeader />}>{children}</SiteFrame>;
}
