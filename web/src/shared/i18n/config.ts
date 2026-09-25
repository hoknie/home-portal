export const locales = ["en", "ru", "es"] as const;

export type Locale = (typeof locales)[number];

export const defaultLocale: Locale = "en";

export function isLocale(value: string | undefined): value is Locale {
  return locales.some((locale) => locale === value);
}

export function builtLocale(value: string | undefined): Locale {
  return isLocale(value) ? value : defaultLocale;
}
