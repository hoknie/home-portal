import { Columns3, FileText, Gauge, Hash, KeyRound, List, type LucideIcon, Minus, MousePointerClick, Rows3, Table, Tag, Type } from "lucide-react";

import type { BlockKind } from "./blocks";

export const BLOCK_ICONS: Record<BlockKind, LucideIcon> = {
  stat: Hash,
  text: Type,
  markdown: FileText,
  list: List,
  table: Table,
  "key-values": KeyRound,
  progress: Gauge,
  badge: Tag,
  button: MousePointerClick,
  divider: Minus,
  row: Columns3,
  column: Rows3,
};
