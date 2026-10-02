"use client";

import { useTranslations } from "next-intl";
import { type ReactNode, useState } from "react";

import type { LibraryEntry } from "@/entities/dashboard";
import type { CustomWidgetData } from "@/entities/widget";
import { ConfirmDialog } from "@/shared/ui/confirm-dialog";
import { FormField } from "@/shared/ui/form-field";
import {
  Button,
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
  Input,
  Tabs,
  TabsContent,
  TabsList,
  TabsTrigger,
} from "@/shared/ui/primitives";

import type { WidgetKind } from "../../model/catalog";
import { AccessTab } from "./access-tab";
import { LookTab } from "./look-tab";

export type DialogTab = "general" | "look" | "access";

export type SettingsDialogProps = {
  entry: LibraryEntry | null;
  kind: WidgetKind | null;
  environments: string[];
  firstTab?: DialogTab;
  renderPreview: (
    entry: LibraryEntry,
    custom: CustomWidgetData | null,
  ) => ReactNode;
  onSave: (entry: LibraryEntry) => Promise<void>;
  onClose: () => void;
};

export function SettingsDialog(props: SettingsDialogProps) {
  const { entry } = props;
  return (
    <Dialog open={entry !== null}>
      {entry ? (
        <Opened
          key={`${entry.id ?? "new"}-${entry.type}`}
          {...props}
          entry={entry}
        />
      ) : null}
    </Dialog>
  );
}

function Opened({
  entry,
  kind,
  environments,
  firstTab,
  renderPreview,
  onSave,
  onClose,
}: SettingsDialogProps & { entry: LibraryEntry }) {
  const t = useTranslations("layoutEditor");
  const [edited, setEdited] = useState<LibraryEntry>(entry);
  const [saving, setSaving] = useState(false);
  const [failure, setFailure] = useState<string | null>(null);
  const [tab, setTab] = useState<DialogTab>(firstTab ?? "general");
  const [leaving, setLeaving] = useState(false);
  const changed = JSON.stringify(edited) !== JSON.stringify(entry);
  const localErrors = kind?.errors?.(edited.settings) ?? {};
  const failing = new Set<DialogTab>(
    Object.keys(localErrors).length > 0 ? ["general"] : [],
  );
  const leave = () => (changed ? setLeaving(true) : onClose());
  const apply = async () => {
    setSaving(true);
    setFailure(null);
    try {
      await onSave(edited);
      onClose();
    } catch (error) {
      setFailure(error instanceof Error ? error.message : String(error));
    } finally {
      setSaving(false);
    }
  };
  const tabs: DialogTab[] = ["general", "look", "access"];
  const Editor = kind?.editor ?? null;
  return (
    <DialogContent
      className="flex max-h-[92dvh] w-full flex-col gap-4 sm:max-w-[1100px] max-sm:h-dvh max-sm:max-h-dvh max-sm:max-w-none max-sm:rounded-none"
      onEscapeKeyDown={(event) => {
        event.preventDefault();
        leave();
      }}
      onPointerDownOutside={(event) => {
        event.preventDefault();
        leave();
      }}
      showCloseButton={false}
      onCloseAutoFocus={(event) => event.preventDefault()}
    >
      <DialogHeader>
        <DialogTitle>{entry.title ?? kind?.title ?? entry.type}</DialogTitle>
        <DialogDescription>
          {entry.id ?? kind?.title ?? entry.type}
        </DialogDescription>
      </DialogHeader>
      <div className="grid min-h-0 flex-1 gap-6 overflow-y-auto lg:grid-cols-[minmax(0,1fr)_minmax(0,26rem)]">
        <section
          aria-label={t("dialog.preview")}
          className="order-first grid content-start gap-2 lg:order-last"
        >
          <h3 className="text-sm font-medium">{t("dialog.preview")}</h3>
          <p className="text-xs text-muted-foreground">
            {t("dialog.previewHint")}
          </p>
          <div
            className="rounded-xl border border-dashed p-3"
            data-dialog-preview=""
          >
            {renderPreview(edited, null)}
          </div>
        </section>
        <Tabs value={tab} onValueChange={(value) => setTab(value as DialogTab)}>
          <TabsList>
            {tabs.map((name) => (
              <TabsTrigger
                key={name}
                value={name}
                aria-label={
                  failing.has(name)
                    ? t("dialog.hasErrors", { tab: t(`dialog.tabs.${name}`) })
                    : undefined
                }
                data-failing={failing.has(name) || undefined}
              >
                {t(`dialog.tabs.${name}`)}
                {failing.has(name) ? (
                  <span
                    aria-hidden
                    className="size-1.5 rounded-full bg-destructive"
                  />
                ) : null}
              </TabsTrigger>
            ))}
          </TabsList>
          <TabsContent value="general" className="grid gap-5">
            <FormField
              id="widget-title"
              label={t("widgetTitle")}
              hint={t("widgetTitleHint")}
              optional
            >
              <Input
                id="widget-title"
                value={edited.title ?? ""}
                onChange={(event) =>
                  setEdited({
                    ...edited,
                    title:
                      event.target.value === "" ? null : event.target.value,
                  })
                }
              />
            </FormField>
            {Editor ? (
              <Editor
                value={edited.settings}
                onChange={(settings) => setEdited({ ...edited, settings })}
              />
            ) : (
              <p className="text-sm text-muted-foreground">
                {t("dialog.settingsInFile")}
              </p>
            )}
          </TabsContent>
          <TabsContent value="look">
            <LookTab
              value={edited.appearance}
              onChange={(appearance) => setEdited({ ...edited, appearance })}
            />
          </TabsContent>
          <TabsContent value="access">
            <AccessTab
              value={{
                environments: edited.environments,
                public: edited.public,
              }}
              environments={environments}
              custom={false}
              onChange={(access) => setEdited({ ...edited, ...access })}
            />
          </TabsContent>
        </Tabs>
      </div>
      <DialogFooter className="flex-row items-center justify-end gap-2">
        {failure ? (
          <p role="alert" className="mr-auto text-sm text-destructive">
            {failure}
          </p>
        ) : null}
        <Button type="button" variant="outline" onClick={leave}>
          {t("dialog.cancel")}
        </Button>
        <Button type="button" onClick={() => void apply()} disabled={saving}>
          {saving ? t("saving") : t("done")}
        </Button>
      </DialogFooter>
      <ConfirmDialog
        open={leaving}
        onOpenChange={setLeaving}
        title={t("dialog.discardTitle")}
        description={t("dialog.discardDescription")}
        confirmLabel={t("dialog.discard")}
        onConfirm={() => {
          setLeaving(false);
          onClose();
        }}
      />
    </DialogContent>
  );
}
