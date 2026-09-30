"use client";

import { useTranslations } from "next-intl";

import { type Coverage, type MatrixRow, type Rights, columnState, matrixState, rowState, withAction, withColumn, withEverything, withRow } from "@/entities/user";
import { ACTIONS, AREAS, type Area } from "@/entities/session";
import { Button } from "@/shared/ui/primitives";

export type MatrixEditorProps = { matrix: MatrixRow[]; rights: Rights; readOnly?: boolean; onChange?: (rights: Rights) => void };

type BulkBoxProps = { label: string; coverage: Coverage; onChange: (on: boolean) => void };

function BulkBox({ label, coverage, onChange }: BulkBoxProps) {
  return (
    <input
      type="checkbox"
      className="size-4 accent-primary"
      aria-label={label}
      aria-checked={coverage === "some" ? "mixed" : coverage === "all"}
      checked={coverage === "all"}
      ref={(box) => {
        if (box) {
          box.indeterminate = coverage === "some";
        }
      }}
      onChange={(event) => onChange(event.target.checked)}
    />
  );
}

export function MatrixEditor({ matrix, rights, readOnly = false, onChange }: MatrixEditorProps) {
  const t = useTranslations("users.groups");
  const label = (area: string) => (AREAS.includes(area as Area) ? t(`areas.${area as Area}`) : area);
  const bulk = !readOnly && onChange !== undefined;
  const everything = matrixState(rights, matrix);
  const offered = (action: string) => matrix.some((row) => row.actions.includes(action));
  return (
    <div className="grid gap-2">
      {bulk ? (
        <div className="flex flex-wrap justify-end gap-2">
          <Button type="button" variant="outline" size="sm" disabled={everything === "all"} onClick={() => onChange(withEverything(true, matrix))}>
            {t("selectAll")}
          </Button>
          <Button type="button" variant="ghost" size="sm" disabled={everything === "none"} onClick={() => onChange(withEverything(false, matrix))}>
            {t("clearAll")}
          </Button>
        </div>
      ) : null}
      <div className="overflow-x-auto">
        <table className="w-full min-w-[32rem] text-sm">
          <thead>
            <tr className="text-left text-xs text-muted-foreground">
              <th className="py-2 pr-3 font-medium">{t("area")}</th>
              {ACTIONS.map((action) => (
                <th key={action} className="px-2 py-2 text-center font-medium">
                  <span className="grid justify-items-center gap-1">
                    {t(`actions.${action}`)}
                    {bulk && offered(action) ? (
                      <BulkBox
                        label={t("everyArea", { action: t(`actions.${action}`) })}
                        coverage={columnState(rights, action, matrix)}
                        onChange={(on) => onChange(withColumn(rights, action, on, matrix))}
                      />
                    ) : null}
                  </span>
                </th>
              ))}
            </tr>
          </thead>
          <tbody>
            {matrix.map((row) => (
              <tr key={row.area} className="border-t">
                <th scope="row" className="py-2 pr-3 text-left font-medium">
                  <span className="flex items-center gap-2">
                    {bulk && row.actions.length > 0 ? (
                      <BulkBox
                        label={t("everyAction", { area: label(row.area) })}
                        coverage={rowState(rights, row.area, matrix)}
                        onChange={(on) => onChange(withRow(rights, row.area, on, matrix))}
                      />
                    ) : null}
                    {label(row.area)}
                  </span>
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
    </div>
  );
}
