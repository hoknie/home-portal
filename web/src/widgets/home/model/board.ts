import type { Portal } from "@/entities/portal";
import type { GridWidget } from "@/features/widget-board";

export function publicGrid(portal: Portal): GridWidget[] {
  return portal.widgets.map((widget, index) => ({
    ...widget,
    key: widget.id ?? `${widget.type}-${index}`,
  }));
}
