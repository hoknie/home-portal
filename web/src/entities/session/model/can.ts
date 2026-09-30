import { useSession } from "./queries";
import { type Area, type Can, canFor, mayOpen } from "./rights";

export function useCan(): Can {
  const session = useSession();
  return canFor(session.data);
}

export function useMayOpen(area: Area): { ready: boolean; allowed: boolean } {
  const session = useSession();
  return { ready: session.data !== undefined || session.isError, allowed: mayOpen(session.data, area) };
}
