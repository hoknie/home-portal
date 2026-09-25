import { routes } from "@/shared/config";
import { Redirect } from "@/shared/ui/redirect";

export default function LegacyServicesPage() {
  return <Redirect to={routes.adminServices} />;
}
