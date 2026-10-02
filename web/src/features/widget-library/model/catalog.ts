import type { LucideIcon } from "lucide-react";
import type { ComponentType } from "react";

import type { ModuleName } from "@/entities/module";

export type SettingsEditorProps = { value: Record<string, unknown>; onChange: (value: Record<string, unknown>) => void };

export type WidgetKind = {
  type: string;
  title: string;
  description: string;
  icon: LucideIcon;
  module: ModuleName | null;
  editor: ComponentType<SettingsEditorProps> | null;
  errors?: (settings: Record<string, unknown>) => Record<string, string>;
};

export type PreviewScope = { environment: string; signedIn: boolean };

export const CUSTOM = "custom";
