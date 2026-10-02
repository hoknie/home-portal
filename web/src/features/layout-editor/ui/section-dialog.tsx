"use client";

import { useTranslations } from "next-intl";
import { useState } from "react";

import { SECTION_SURFACES, type Section, TITLE_VISIBILITIES } from "@/shared/api";
import { FormField } from "@/shared/ui/form-field";
import { Button, Dialog, DialogContent, DialogFooter, DialogHeader, DialogTitle, Input } from "@/shared/ui/primitives";

import { ChoiceGroup } from "@/shared/ui/choice-group";

export type SectionDialogProps = { section: Section | null; onApply: (patch: Partial<Omit<Section, "id">>) => void; onClose: () => void };

function Opened({ section, onApply, onClose }: { section: Section; onApply: SectionDialogProps["onApply"]; onClose: () => void }) {
  const t = useTranslations("layoutEditor");
  const [edited, setEdited] = useState(section);
  return (
    <DialogContent className="sm:max-w-lg">
      <DialogHeader>
        <DialogTitle>{t("section.settings")}</DialogTitle>
      </DialogHeader>
      <div className="grid gap-5">
        <FormField id="section-title" label={t("section.title")} optional>
          <Input id="section-title" value={edited.title ?? ""} onChange={(event) => setEdited({ ...edited, title: event.target.value.trim() === "" ? null : event.target.value })} />
        </FormField>
        <ChoiceGroup
          label={t("section.titleShown")}
          value={edited.appearance.title}
          onChange={(title) => setEdited({ ...edited, appearance: { ...edited.appearance, title } })}
          choices={TITLE_VISIBILITIES.map((value) => ({ value, label: t(`section.titles.${value}`) }))}
        />
        <ChoiceGroup
          label={t("section.surface")}
          value={edited.appearance.surface}
          onChange={(surface) => setEdited({ ...edited, appearance: { ...edited.appearance, surface } })}
          choices={SECTION_SURFACES.map((value) => ({ value, label: t(`section.surfaces.${value}`) }))}
        />
      </div>
      <DialogFooter className="flex-row justify-end gap-2">
        <Button type="button" variant="outline" onClick={onClose}>
          {t("dialog.cancel")}
        </Button>
        <Button
          type="button"
          onClick={() => {
            onApply({ title: edited.title, appearance: edited.appearance });
            onClose();
          }}
        >
          {t("done")}
        </Button>
      </DialogFooter>
    </DialogContent>
  );
}

export function SectionDialog({ section, onApply, onClose }: SectionDialogProps) {
  return (
    <Dialog open={section !== null} onOpenChange={(next) => (next ? undefined : onClose())}>
      {section ? <Opened key={section.id} section={section} onApply={onApply} onClose={onClose} /> : null}
    </Dialog>
  );
}
