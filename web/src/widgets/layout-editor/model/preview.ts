import type { Service, ServiceView } from "@/entities/service";
import type { Draft, PreviewScope } from "@/features/layout-editor";
import type { GridWidget } from "@/features/widget-board";

export function previewWidgets(draft: Draft, scope: PreviewScope): GridWidget[] {
  return draft.widgets
    .filter((widget) => widget.environments === null || widget.environments.includes(scope.environment))
    .filter((widget) => scope.signedIn || widget.public)
    .map((widget) => ({ ...widget, key: widget.uid }));
}

export function previewServices(services: Service[], scope: PreviewScope): ServiceView[] {
  return services
    .filter((service) => service.environments === null || service.environments.includes(scope.environment))
    .filter((service) => scope.signedIn || service.public)
    .map((service) => ({
      ...service,
      address: service.addresses[scope.environment] ?? service.url,
      status: scope.signedIn || service.public_status ? service.status : null,
    }));
}
