"use client";

import type { ReactNode } from "react";

import { useSession } from "@/entities/session";
import { PageTransition, Skeleton } from "@/shared/ui/kit";

import { AppShell } from "./app-shell";

export type SiteFrameProps = { guest: ReactNode; children: ReactNode };

export function SiteFrame({ guest, children }: SiteFrameProps) {
  const session = useSession();
  if (session.data) {
    return <AppShell redirectGuests={false}>{children}</AppShell>;
  }
  if (!session.isPending) {
    return (
      <div className="mx-auto grid w-full max-w-6xl gap-8 px-4 py-6 sm:px-6 lg:py-10">
        {guest}
        <PageTransition>{children}</PageTransition>
      </div>
    );
  }
  return (
    <div className="flex min-h-svh items-center justify-center" aria-busy="true">
      <Skeleton className="h-10 w-48" />
    </div>
  );
}
