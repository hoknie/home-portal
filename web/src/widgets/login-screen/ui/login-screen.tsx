"use client";

import { House } from "lucide-react";
import { useRouter, useSearchParams } from "next/navigation";
import { useTranslations } from "next-intl";
import { useCallback, useEffect, useRef } from "react";

import { LanguageSwitch } from "@/features/language-switch";
import { SignInForm } from "@/features/sign-in";
import { useSession } from "@/entities/session";
import { NEXT_PARAMETER } from "@/shared/api";
import { RETURN_PARAMETER, destinationOf, leaveTo } from "@/shared/lib/navigation";
import { Card, CardContent, CardDescription, CardHeader, CardTitle, Skeleton } from "@/shared/ui/primitives";

export function LoginScreen() {
  const t = useTranslations();
  const router = useRouter();
  const parameters = useSearchParams();
  const destination = destinationOf(parameters.get(NEXT_PARAMETER), parameters.get(RETURN_PARAMETER));
  const session = useSession();
  const signedIn = Boolean(session.data);
  const gone = useRef(false);
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
    if (signedIn) {
      go();
    }
  }, [signedIn, go]);

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
          {session.isPending || signedIn ? <Skeleton className="h-48 w-full" aria-busy="true" /> : <SignInForm onSignedIn={go} />}
        </CardContent>
      </Card>
    </main>
  );
}
