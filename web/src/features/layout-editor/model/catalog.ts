import type { ComponentType } from "react";

export type SettingsEditorProps = { value: Record<string, unknown>; onChange: (value: Record<string, unknown>) => void };

export type WidgetKind = { type: string; title: string; editor: ComponentType<SettingsEditorProps> | null };

export type PreviewScope = { environment: string; signedIn: boolean };
