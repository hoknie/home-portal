"use client";

import dynamic from "next/dynamic";

import { Skeleton } from "@/shared/ui/kit";

export const CanvasLoader = dynamic(() => import("./flow-canvas"), {
  ssr: false,
  loading: () => <Skeleton className="size-full" aria-busy="true" />,
});
