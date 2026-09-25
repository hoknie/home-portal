import type { Locale } from "@/shared/i18n";

export const LANGUAGE_COOKIE = "portal_language";
export const LANGUAGE_SECONDS = 60 * 60 * 24 * 365;

export const page = {
  reload: () => window.location.reload(),
};

export function languageCookie(locale: Locale) {
  return `${LANGUAGE_COOKIE}=${locale}; Path=/; Max-Age=${LANGUAGE_SECONDS}; SameSite=Lax`;
}

export function chooseLanguage(locale: Locale) {
  document.cookie = languageCookie(locale);
  page.reload();
}
