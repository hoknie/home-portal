"use client";

import { TriangleAlert } from "lucide-react";
import { useRouter } from "next/navigation";
import { useTranslations } from "next-intl";
import { useEffect } from "react";

import { LanguageSwitch } from "@/features/language-switch";
import { useFailure } from "@/entities/portal";
import { routes } from "@/shared/config";
import { Card, CardContent, CardDescription, CardHeader, CardTitle, Skeleton } from "@/shared/ui/primitives";
import { RelativeTime } from "@/shared/ui/relative-time";

import { ProblemList } from "./problem-list";

export function FatalScreen() {
  const t = useTranslations("fatal");
  const router = useRouter();
  const failure = useFailure();
  const running = failure.data?.kind === "running";
  const report = failure.data?.kind === "failed" ? failure.data.report : null;

  useEffect(() => {
    if (running) {
      router.replace(routes.home);
    }
  }, [running, router]);

  return (
    <main className="relative flex min-h-svh items-center justify-center px-4 py-16">
      <div className="absolute top-4 right-4">
        <LanguageSwitch />
      </div>
      <Card className="w-full max-w-2xl">
        <CardHeader>
          <span className="mb-2 flex size-11 items-center justify-center rounded-xl bg-destructive/10 text-destructive">
            <TriangleAlert className="size-5" aria-hidden />
          </span>
          <CardTitle className="text-xl">{t("title")}</CardTitle>
          <CardDescription>{report?.details ? t("detailsLead") : t("hiddenLead")}</CardDescription>
        </CardHeader>
        <CardContent className="grid gap-4">
          {failure.isError ? (
            <p role="status" className="text-sm text-muted-foreground">
              {t("waiting")}
            </p>
          ) : null}
          {running ? (
            <p role="status" className="text-sm text-muted-foreground">
              {t("running")}
            </p>
          ) : null}
          {!report && !running && !failure.isError ? <Skeleton className="h-32 w-full" aria-busy="true" /> : null}
          {report?.details ? (
            <>
              <ProblemList problems={report.problems ?? []} />
              <p className="text-sm text-muted-foreground">
                {t("recovers")} {t("checked")} <RelativeTime moment={report.checked} />
              </p>
            </>
          ) : null}
          {report && !report.details ? (
            <div className="grid gap-2 text-sm text-muted-foreground">
              <p>{t("hiddenHint")}</p>
              <ul className="grid gap-1">
                <li>
                  {t("logMac")} <code className="font-mono">{t("logMacPath")}</code>
                </li>
                <li>
                  {t("logLinux")} <code className="font-mono">{t("logLinuxCommand")}</code>
                </li>
              </ul>
            </div>
          ) : null}
        </CardContent>
      </Card>
    </main>
  );
}
