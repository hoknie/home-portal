import { emptySchema, request } from "@/shared/api";
import { api } from "@/shared/config";

import { type LibraryEntry, libraryWidgetSchema, librarySchema } from "../model/library";

export function fetchLibrary() {
  return request(api.library, { schema: librarySchema });
}

export function saveLibraryWidget(entry: LibraryEntry, existing: string | null, revision: string | null) {
  const path = existing === null ? api.library : api.libraryWidget(existing);
  return request(path, { method: existing === null ? "POST" : "PUT", body: entry, revision, schema: libraryWidgetSchema });
}

export function deleteLibraryWidget(id: string, revision: string | null) {
  return request(api.libraryWidget(id), { method: "DELETE", revision, schema: emptySchema });
}
