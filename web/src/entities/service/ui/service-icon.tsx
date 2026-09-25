"use client";

import { Server } from "lucide-react";
import { DynamicIcon, type IconName, iconNames } from "lucide-react/dynamic";
import Image from "next/image";
import { useState } from "react";

import { cn } from "@/shared/lib/cn";

const KNOWN = new Set<string>(iconNames);

export type ServiceIconProps = { name: string | null; source?: string | null; size?: "sm" | "md" };

const SIZE = { sm: "size-4", md: "size-5" } as const;

export function ServiceIcon({ name, source = null, size = "md" }: ServiceIconProps) {
  const [broken, setBroken] = useState(false);
  const className = SIZE[size];
  if (source && !broken) {
    return (
      <Image
        src={source}
        alt=""
        aria-hidden
        width={20}
        height={20}
        unoptimized
        className={cn(className, "object-contain")}
        onError={() => setBroken(true)}
      />
    );
  }
  if (!name || !KNOWN.has(name)) {
    return <Server className={className} aria-hidden />;
  }
  return <DynamicIcon name={name as IconName} className={className} fallback={() => <Server className={className} aria-hidden />} aria-hidden />;
}
