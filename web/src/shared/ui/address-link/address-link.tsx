"use client";

import type { ComponentProps, MouseEvent } from "react";

import { ADDRESS_LINK_ATTRIBUTE, pushAddress } from "@/shared/lib/navigation";

export type AddressLinkProps = Omit<ComponentProps<"a">, "href"> & { href: string };

const MAIN_BUTTON = 0;

export function AddressLink({ href, onClick, ...props }: AddressLinkProps) {
  const follow = (event: MouseEvent<HTMLAnchorElement>) => {
    onClick?.(event);
    if (event.defaultPrevented || event.button !== MAIN_BUTTON || event.metaKey || event.ctrlKey || event.shiftKey || event.altKey || props.target === "_blank") {
      return;
    }
    event.preventDefault();
    pushAddress(href);
  };
  return <a href={href} onClick={follow} {...{ [ADDRESS_LINK_ATTRIBUTE]: "" }} {...props} />;
}
