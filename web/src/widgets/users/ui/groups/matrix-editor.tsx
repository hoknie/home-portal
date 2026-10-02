"use client";

import { useTranslations } from "next-intl";

import { type Coverage, type MatrixRow, type Rights, columnState, matrixState, rowState, withAction, withColumn, withEverything, withRow } from "@/entities/user";
import { ACTIONS, AREAS, type Area } from "@/entities/session";
import { Button, Checkbox, Table, TableBody, TableCell, TableHead, TableHeader, TableRow } from "@/shared/ui/kit";

export type MatrixEditorProps = { matrix: MatrixRow[]; rights: Rights; readOnly?: boolean; onChange?: (rights: Rights) => void };

type BulkBoxProps = { label: string; coverage: Coverage; onChange: (on: boolean) => void };

function BulkBox({ label, coverage, onChange }: BulkBoxProps) {
  return (
    <Checkbox
      aria-label={label}
      checked={coverage === "all" ? true : coverage === "some" ? "indeterminate" : false}
      onCheckedChange={(checked) => onChange(checked === true)}
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
      <Table className="min-w-[32rem]">
          <TableHeader>
            <TableRow className="text-left text-xs text-muted-foreground hover:bg-transparent">
              <TableHead className="h-auto px-0 py-2 pr-3 text-xs text-muted-foreground">{t("area")}</TableHead>
              {ACTIONS.map((action) => (
                <TableHead key={action} className="h-auto px-2 py-2 text-center text-xs text-muted-foreground">
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
                </TableHead>
              ))}
            </TableRow>
          </TableHeader>
          <TableBody>
            {matrix.map((row) => (
              <TableRow key={row.area} className="hover:bg-transparent">
                <TableHead scope="row" className="h-auto px-0 py-2 pr-3 text-left">
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
                </TableHead>
                {ACTIONS.map((action) => (
                  <TableCell key={action} className="px-2 py-2 text-center">
                    {row.actions.includes(action) ? (
                      <Checkbox
                        aria-label={t("cell", { area: label(row.area), action: t(`actions.${action}`) })}
                        checked={rights[row.area]?.includes(action) ?? false}
                        disabled={readOnly}
                        onCheckedChange={(checked) => onChange?.(withAction(rights, row.area, action, checked === true, matrix))}
                      />
                    ) : null}
                  </TableCell>
                ))}
              </TableRow>
            ))}
          </TableBody>
      </Table>
    </div>
  );
}
