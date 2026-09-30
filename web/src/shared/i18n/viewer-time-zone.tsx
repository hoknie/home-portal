"use client";

import { NextIntlClientProvider, useLocale, useMessages, useTimeZone } from "next-intl";
import { type ReactNode, useSyncExternalStore } from "react";

export function browserTimeZone(): string | undefined {
  try {
    return Intl.DateTimeFormat().resolvedOptions().timeZone || undefined;
  } catch {
    return undefined;
  }
}

const unchanging = () => () => undefined;

export function ViewerTimeZone({ children }: { children: ReactNode }) {
  const locale = useLocale();
  const messages = useMessages();
  const built = useTimeZone();
  const zone = useSyncExternalStore(unchanging, () => browserTimeZone() ?? built, () => built);
  return (
    <NextIntlClientProvider locale={locale} messages={messages} timeZone={zone}>
      {children}
    </NextIntlClientProvider>
  );
}
