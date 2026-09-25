import type { ReactNode } from "react";

import { cn } from "@/shared/lib/cn";
import { Table, TableBody, TableCell, TableHead, TableHeader, TableRow } from "@/shared/ui/primitives";

export type Column<Row> = {
  key: string;
  header: string;
  cell: (row: Row) => ReactNode;
  align?: "start" | "end";
  hideBelow?: "sm" | "md" | "lg";
};

export type DataTableProps<Row> = {
  columns: Column<Row>[];
  rows: Row[];
  rowKey: (row: Row) => string;
  empty?: ReactNode;
};

const HIDE_BELOW = { sm: "hidden sm:table-cell", md: "hidden md:table-cell", lg: "hidden lg:table-cell" } as const;

function columnClass<Row>(column: Column<Row>) {
  return cn(column.align === "end" && "text-right", column.hideBelow && HIDE_BELOW[column.hideBelow]);
}

export function DataTable<Row>({ columns, rows, rowKey, empty }: DataTableProps<Row>) {
  if (rows.length === 0 && empty) {
    return <>{empty}</>;
  }
  return (
    <Table>
      <TableHeader>
        <TableRow>
          {columns.map((column) => (
            <TableHead key={column.key} className={cn("px-4", columnClass(column))}>
              {column.header}
            </TableHead>
          ))}
        </TableRow>
      </TableHeader>
      <TableBody>
        {rows.map((row) => (
          <TableRow key={rowKey(row)}>
            {columns.map((column) => (
              <TableCell key={column.key} className={cn("px-4", columnClass(column))}>
                {column.cell(row)}
              </TableCell>
            ))}
          </TableRow>
        ))}
      </TableBody>
    </Table>
  );
}
