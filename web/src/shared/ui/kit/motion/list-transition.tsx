"use client";

import { Fragment, type ReactNode, useDeferredValue } from "react";

import { Boundary } from "./boundary";

export const ANIMATED_ITEMS = 40;

export type ListTransitionProps<Item> = {
  items: readonly Item[];
  keyOf: (item: Item) => string;
  children: (item: Item) => ReactNode;
};

export function ListTransition<Item>({ items, keyOf, children }: ListTransitionProps<Item>) {
  const shown = useDeferredValue(items);
  if (shown.length > ANIMATED_ITEMS) {
    return (
      <>
        {shown.map((item) => (
          <Fragment key={keyOf(item)}>{children(item)}</Fragment>
        ))}
      </>
    );
  }
  return (
    <>
      {shown.map((item) => (
        <Boundary key={keyOf(item)} enter="list-enter" exit="list-exit" update="list-move" default="none">
          {children(item)}
        </Boundary>
      ))}
    </>
  );
}
