import { type ComponentType, type ReactNode, type ViewTransitionProps, ViewTransition } from "react";

function Passthrough({ children }: { children?: ReactNode }) {
  return <>{children}</>;
}

export const Boundary: ComponentType<ViewTransitionProps> = (ViewTransition as ComponentType<ViewTransitionProps> | undefined) ?? Passthrough;
