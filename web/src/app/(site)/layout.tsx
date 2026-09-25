import type { ReactNode } from "react";

import { SiteHeader } from "@/widgets/site-header";

export default function SiteLayout({ children }: { children: ReactNode }) {
  return (
    <div className="mx-auto grid w-full max-w-6xl gap-8 px-4 py-6 sm:px-6 lg:py-10">
      <SiteHeader />
      {children}
    </div>
  );
}
