import { NextIntlClientProvider } from "next-intl";
import type { ReactNode } from "react";

import type { Locale } from "./config";
import { dictionaries } from "./messages";

export const TEST_LOCALE: Locale = "ru";

export function TestIntl({ children, locale = TEST_LOCALE }: { children: ReactNode; locale?: Locale }) {
  return (
    <NextIntlClientProvider locale={locale} messages={dictionaries[locale]} timeZone="UTC">
      {children}
    </NextIntlClientProvider>
  );
}
