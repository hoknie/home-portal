import { X } from "lucide-react";
import { useTranslations } from "next-intl";
import type { ReactNode } from "react";

import { AddressLink } from "@/shared/ui/address-link";
import { buttonVariants } from "@/shared/ui/primitives";

export type SideColumnProps = { title: string; actions?: ReactNode; closeHref?: string; children: ReactNode };

export function CloseLink({ href }: { href: string }) {
  const t = useTranslations("common");
  return (
    <AddressLink href={href} aria-label={t("close")} title={t("close")} className={buttonVariants({ variant: "ghost", size: "icon" })}>
      <X aria-hidden />
    </AddressLink>
  );
}

export function CloseButton({ onClose }: { onClose: () => void }) {
  const t = useTranslations("common");
  return (
    <button type="button" aria-label={t("close")} title={t("close")} onClick={onClose} className={buttonVariants({ variant: "ghost", size: "icon" })}>
      <X aria-hidden />
    </button>
  );
}

export function SideColumn({ title, actions, closeHref, children }: SideColumnProps) {
  return (
    <aside aria-label={title} className="glass-panel flex max-h-[45%] min-h-0 w-full shrink-0 flex-col overflow-hidden rounded-2xl md:max-h-full md:w-[26rem]">
      <header className="flex flex-wrap items-center gap-2 border-b border-glass-edge p-4">
        <h2 className="text-base font-semibold">{title}</h2>
        {actions || closeHref ? (
          <span className="ms-auto flex items-center gap-1">
            {actions}
            {closeHref ? <CloseLink href={closeHref} /> : null}
          </span>
        ) : null}
      </header>
      <div className="min-h-0 flex-1 overflow-y-auto">{children}</div>
    </aside>
  );
}
