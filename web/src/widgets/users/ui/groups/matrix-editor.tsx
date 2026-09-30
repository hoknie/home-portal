"use client";

import { useTranslations } from "next-intl";

import { type MatrixRow, type Rights, withAction } from "@/entities/user";
import { ACTIONS, AREAS, type Area } from "@/entities/session";

export type MatrixEditorProps = { matrix: MatrixRow[]; rights: Rights; readOnly?: boolean; onChange?: (rights: Rights) => void };

export function MatrixEditor({ matrix, rights, readOnly = false, onChange }: MatrixEditorProps) {
  const t = useTranslations("users.groups");
  const label = (area: string) => (AREAS.includes(area as Area) ? t(`areas.${area as Area}`) : area);
  return (
    <div className="overflow-x-auto">
      <table className="w-full min-w-[32rem] text-sm">
        <thead>
          <tr className="text-left text-xs text-muted-foreground">
            <th className="py-2 pr-3 font-medium">{t("area")}</th>
            {ACTIONS.map((action) => (
              <th key={action} className="px-2 py-2 text-center font-medium">
                {t(`actions.${action}`)}
              </th>
            ))}
          </tr>
        </thead>
        <tbody>
          {matrix.map((row) => (
            <tr key={row.area} className="border-t">
              <th scope="row" className="py-2 pr-3 text-left font-medium">
                {label(row.area)}
              </th>
              {ACTIONS.map((action) => (
                <td key={action} className="px-2 py-2 text-center">
                  {row.actions.includes(action) ? (
                    <input
                      type="checkbox"
                      className="size-4 accent-primary"
                      aria-label={t("cell", { area: label(row.area), action: t(`actions.${action}`) })}
                      checked={rights[row.area]?.includes(action) ?? false}
                      disabled={readOnly}
                      onChange={(event) => onChange?.(withAction(rights, row.area, action, event.target.checked, matrix))}
                    />
                  ) : null}
                </td>
              ))}
            </tr>
          ))}
        </tbody>
      </table>
    </div>
  );
}
