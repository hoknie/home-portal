"use client";

import { House } from "lucide-react";
import { useRouter, useSearchParams } from "next/navigation";
import { useTranslations } from "next-intl";
import { useCallback, useEffect, useRef, useState } from "react";

import { LanguageSwitch } from "@/features/language-switch";
import { SignInForm } from "@/features/sign-in";
import { useSession } from "@/entities/session";
import { NEXT_PARAMETER } from "@/shared/api";
import { routes } from "@/shared/config";
import { RETURN_PARAMETER, cameBackFrom, destinationOf, hostOf, leaveTo } from "@/shared/lib/navigation";
import { ErrorNotice } from "@/shared/ui/error-notice";
import { Button, Card, CardContent, CardDescription, CardHeader, CardTitle, Skeleton } from "@/shared/ui/primitives";

export function LoginScreen() {
  const t = useTranslations();
  const router = useRouter();
  const parameters = useSearchParams();
  const destination = destinationOf(parameters.get(NEXT_PARAMETER), parameters.get(RETURN_PARAMETER));
  const session = useSession();
  const signedIn = Boolean(session.data);
  const gone = useRef(false);
  const [looping] = useState(() => destination.leavesTheInterface && cameBackFrom(destination.href));
  const go = useCallback(() => {
    if (gone.current) {
      return;
    }
    gone.current = true;
    if (destination.leavesTheInterface) {
      leaveTo(destination.href);
    } else {
      router.replace(destination.href);
    }
  }, [destination.href, destination.leavesTheInterface, router]);

  useEffect(() => {
    if (signedIn && !looping) {
      go();
    }
  }, [signedIn, looping, go]);

  const retry = () => {
    gone.current = true;
    leaveTo(destination.href);
  };

  return (
    <main className="relative flex min-h-svh items-center justify-center px-4">
      <div className="absolute top-4 right-4">
        <LanguageSwitch />
      </div>
      <Card className="w-full max-w-sm">
        <CardHeader className="items-center text-center">
          <span className="mx-auto mb-2 flex size-11 items-center justify-center rounded-xl bg-primary text-primary-foreground shadow">
            <House className="size-5" aria-hidden />
          </span>
          <CardTitle className="text-xl">{t("login.title")}</CardTitle>
          <CardDescription>{t("login.subtitle")}</CardDescription>
        </CardHeader>
        <CardContent>
          {looping && signedIn ? (
            <ErrorNotice
              title={t("login.loop.title", { host: hostOf(parameters.get(RETURN_PARAMETER)) })}
              description={t("login.loop.description")}
              action={
                <div className="flex flex-wrap gap-2">
                  <Button size="sm" onClick={retry}>
                    {t("login.loop.retry")}
                  </Button>
                  <Button size="sm" variant="outline" onClick={() => router.replace(routes.home)}>
                    {t("login.loop.home")}
                  </Button>
                </div>
              }
            />
          ) : session.isPending || signedIn ? (
            <Skeleton className="h-48 w-full" aria-busy="true" />
          ) : (
            <SignInForm onSignedIn={go} />
          )}
        </CardContent>
      </Card>
    </main>
  );
}
