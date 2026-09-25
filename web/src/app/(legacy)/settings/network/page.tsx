import { routes } from "@/shared/config";
import { Redirect } from "@/shared/ui/redirect";

export default function LegacyNetworkPage() {
  return <Redirect to={routes.adminNetwork} />;
}
