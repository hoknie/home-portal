import { render, screen } from "@testing-library/react";
import { expect, it } from "vitest";

import { Table, TableBody, TableCell, TableHead, TableHeader, TableRow } from "./table";

it("renders an accessible table", () => {
  render(
    <Table>
      <TableHeader>
        <TableRow>
          <TableHead>name</TableHead>
        </TableRow>
      </TableHeader>
      <TableBody>
        <TableRow>
          <TableCell>nas</TableCell>
        </TableRow>
      </TableBody>
    </Table>,
  );
  expect(screen.getByRole("columnheader", { name: "name" })).toBeInTheDocument();
  expect(screen.getByRole("cell", { name: "nas" })).toBeInTheDocument();
  expect(screen.getByRole("cell", { name: "nas" }).closest("tr")).toHaveClass("hover:bg-glass-tint");
});
