import { render, screen } from "@testing-library/react";
import { expect, it } from "vitest";

import { DataTable } from "./data-table";

type Row = { id: string; name: string };

const columns = [
  { key: "name", header: "Name", cell: (row: Row) => row.name },
  { key: "id", header: "Id", cell: (row: Row) => row.id, align: "end" as const, hideBelow: "md" as const },
];

it("renders one row per item under the column headers", () => {
  render(<DataTable columns={columns} rows={[{ id: "a", name: "Alpha" }, { id: "b", name: "Beta" }]} rowKey={(row) => row.id} />);
  expect(screen.getByRole("columnheader", { name: "Name" })).toBeInTheDocument();
  expect(screen.getAllByRole("row")).toHaveLength(3);
  expect(screen.getByRole("cell", { name: "b" })).toHaveClass("text-right", "hidden", "md:table-cell");
});

it("shows the empty slot instead of an empty table", () => {
  render(<DataTable columns={columns} rows={[]} rowKey={(row) => row.id} empty={<p>none</p>} />);
  expect(screen.getByText("none")).toBeInTheDocument();
  expect(screen.queryByRole("table")).not.toBeInTheDocument();
});
