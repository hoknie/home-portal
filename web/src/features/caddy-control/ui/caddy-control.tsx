"use client";

import { Download, Play, Square } from "lucide-react";
import { useTranslations } from "next-intl";
import { toast } from "sonner";

import { CADDY_LATEST, type Caddy, type Proxy, useDownloadCaddy, useStartCaddy, useStopCaddy } from "@/entities/proxy";
import { RequestError } from "@/shared/api";
import { KvList, KvRow } from "@/shared/ui/kv-list";
import { Button } from "@/shared/ui/primitives";
import { SectionCard } from "@/shared/ui/section-card";

import { CaddySourceForm } from "./caddy-source-form";

export type CaddyControlProps = { proxy: Proxy; revision: string | null };

const ANY_VERSION = "*";

function archiveOf(caddy: Caddy) {
  const version = caddy.version === CADDY_LATEST ? ANY_VERSION : caddy.version;
  return caddy.platform === null ? null : `caddy_${version}_${caddy.platform}.tar.gz`;
}

export function CaddyControl({ proxy, revision }: CaddyControlProps) {
  const t = useTranslations("proxy.caddyControl");
  const download = useDownloadCaddy();
  const start = useStartCaddy();
  const stop = useStopCaddy();
  const caddy = proxy.caddy;
  const downloading = caddy.download.state === "downloading" || download.isPending;
  const busy = downloading || start.isPending || stop.isPending;
  const failed = (error: unknown) => {
    const message = error instanceof RequestError && error.message !== "" ? error.message : t("unknownError");
    toast.error(t("failed", { message }));
  };
  const run = async (action: () => Promise<unknown>, done: string) => {
    try {
      await action();
      toast.success(done);
    } catch (error) {
      failed(error);
    }
  };
  const silent = !proxy.reachable && caddy.managed && caddy.log.length > 0;
  const archive = archiveOf(caddy);
  const latest = caddy.version === CADDY_LATEST;
  return (
    <SectionCard title={t("title")} description={t("description")}>
      <div className="grid gap-4">
        <KvList>
          <KvRow label={t("version")}>{caddy.installed === null ? t("notInstalled") : caddy.installed || t("unknownVersion")}</KvRow>
          {caddy.installed !== null ? (
            <KvRow label={t("installedFrom")}>
              {caddy.installed_from ? <span className="font-mono text-xs break-all">{caddy.installed_from}</span> : t("unknownOrigin")}
            </KvRow>
          ) : null}
          <KvRow label={t("nextDownload")}>
            <span className="font-mono text-xs break-all">{caddy.release_url}</span>
          </KvRow>
          <KvRow label={t("archive")}>
            {archive ? (
              <span className="font-mono text-xs break-all">{archive}</span>
            ) : (
              <span role="alert" className="text-status-down">
                {caddy.platform_error ?? t("unsupported")}
              </span>
            )}
          </KvRow>
          <KvRow label={t("verification")}>{t("verificationNote")}</KvRow>
          <KvRow label={t("mode")}>{caddy.managed ? t("managed") : t("external")}</KvRow>
          {downloading ? (
            <KvRow label={t("download")}>
              <span role="status">{t("downloading")}</span>
            </KvRow>
          ) : null}
          {caddy.download.state === "failed" && caddy.download.error ? (
            <KvRow label={t("download")}>
              <span role="alert" className="text-status-down">
                {caddy.download.error}
              </span>
            </KvRow>
          ) : null}
        </KvList>
        <div className="flex flex-wrap gap-2">
          <Button
            type="button"
            variant="outline"
            size="sm"
            disabled={busy || archive === null}
            onClick={() => void run(() => download.mutateAsync(), t("downloadStarted"))}
          >
            <Download aria-hidden />
            {latest ? t("downloadLatest") : t("downloadVersion", { version: caddy.version })}
          </Button>
          {caddy.managed ? (
            <Button type="button" variant="outline" size="sm" disabled={busy} onClick={() => void run(() => stop.mutateAsync(revision), t("stopped"))}>
              <Square aria-hidden />
              {t("stop")}
            </Button>
          ) : (
            <Button type="button" size="sm" disabled={busy || caddy.installed === null} onClick={() => void run(() => start.mutateAsync(revision), t("started"))}>
              <Play aria-hidden />
              {t("start")}
            </Button>
          )}
        </div>
        {silent ? (
          <div className="grid gap-2">
            <p className="text-sm text-muted-foreground">{t("logHint")}</p>
            <pre aria-label={t("log")} className="max-h-64 overflow-auto rounded-lg border bg-muted/40 p-3 font-mono text-xs whitespace-pre-wrap">
              {caddy.log.join("\n")}
            </pre>
          </div>
        ) : null}
        <CaddySourceForm proxy={proxy} revision={revision} />
      </div>
    </SectionCard>
  );
}
