import { getRequestConfig } from "next-intl/server";

import { builtLocale } from "./config";
import { dictionaries } from "./messages";

export default getRequestConfig(async () => {
  const locale = builtLocale(process.env.PORTAL_LANGUAGE);
  return {
    locale,
    messages: dictionaries[locale],
    timeZone: "Europe/Moscow",
  };
});
