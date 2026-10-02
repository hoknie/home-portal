"use client";

import { useSession } from "@/entities/session";

import { BoardSkeleton } from "./board-state";
import { PublicBoard } from "./public-board";
import { SignedInBoard } from "./signed-in-board";

export function HomeScreen() {
  const session = useSession();
  if (session.isPending) {
    return <BoardSkeleton />;
  }
  return session.data ? <SignedInBoard /> : <PublicBoard />;
}
