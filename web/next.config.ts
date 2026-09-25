import type { NextConfig } from "next";
import { PHASE_DEVELOPMENT_SERVER } from "next/constants";
import createNextIntlPlugin from "next-intl/plugin";

const withNextIntl = createNextIntlPlugin("./src/shared/i18n/request.ts");

const portalOrigin = process.env.HOME_PORTAL_ORIGIN ?? "http://127.0.0.1:8080";

const shared: NextConfig = {
  trailingSlash: true,
  images: { unoptimized: true },
};

export default function config(phase: string): NextConfig {
  if (phase === PHASE_DEVELOPMENT_SERVER) {
    return withNextIntl({
      ...shared,
      async rewrites() {
        return [{ source: "/api/:path*", destination: `${portalOrigin}/api/:path*` }];
      },
    });
  }
  return withNextIntl({ ...shared, output: "export" });
}
