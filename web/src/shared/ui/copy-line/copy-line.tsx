"use client";

import { Copy } from "lucide-react";
import { useTranslations } from "next-intl";
import { toast } from "sonner";

import { Button } from "@/shared/ui/primitives";

export type CopyLineProps = { label?: string; text: string };

export function CopyLine({ label, text }: CopyLineProps) {
  const t = useTranslations("common");
  const copy = async () => {
    try {
      await navigator.clipboard.writeText(text);
      toast.success(t("copied"));
    } catch {
      toast.error(t("copyFailed"));
    }
  };
  return (
    <div className="grid gap-1">
      {label ? <p className="text-sm font-medium">{label}</p> : null}
      <div className="flex items-start gap-2">
        <code className="flex-1 rounded-md border border-glass-edge bg-glass-tint px-2 py-1.5 font-mono text-xs break-all">{text}</code>
        <Button type="button" variant="outline" size="icon" aria-label={t("copy")} onClick={() => void copy()}>
          <Copy aria-hidden />
        </Button>
      </div>
    </div>
  );
}
