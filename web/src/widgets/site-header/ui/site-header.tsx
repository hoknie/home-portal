"use client";

import { useEnvironment } from "@/entities/environment";
import { usePortal } from "@/entities/portal";
import { useSession } from "@/entities/session";
import { Skeleton } from "@/shared/ui/primitives";

import { SiteHeaderBar } from "./site-header-bar";

export function SiteHeader() {
  const session = useSession();
  if (session.isPending) {
    return <Skeleton className="h-15 w-full rounded-xl" aria-busy="true" />;
  }
  return session.data ? <SignedInHeader user={session.data.name} /> : <PublicHeader />;
}

type Reported = { environment: string; detected: string; switchable: boolean; environments: string[] } | undefined;

function switchOf(reported: Reported) {
  return {
    environment: reported?.environment ?? null,
    detected: reported?.detected ?? null,
    switchable: reported?.switchable ?? false,
    environments: reported?.environments ?? [],
  };
}

function SignedInHeader({ user }: { user: string }) {
  const environment = useEnvironment();
  return <SiteHeaderBar environment={switchOf(environment.data)} user={user} />;
}

function PublicHeader() {
  const portal = usePortal();
  return <SiteHeaderBar environment={switchOf(portal.data)} user={null} />;
}
