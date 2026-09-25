"use client";

import { useSession } from "@/entities/session";
import { Skeleton } from "@/shared/ui/primitives";

import { PublicBoard } from "./public-board";
import { SignedInBoard } from "./signed-in-board";

export function HomeScreen() {
  const session = useSession();
  if (session.isPending) {
    return (
      <div className="grid gap-4" aria-busy="true">
        <Skeleton className="h-48 w-full" />
      </div>
    );
  }
  return session.data ? <SignedInBoard /> : <PublicBoard />;
}
