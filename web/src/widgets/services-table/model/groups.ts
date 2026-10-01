export type Grouped<Item> = { name: string | null; items: Item[] };

export function byGroup<Item extends { group: string | null }>(items: readonly Item[], locale: string): Grouped<Item>[] {
  const groups = new Map<string | null, Item[]>();
  for (const item of items) {
    const name = item.group?.trim() ? item.group.trim() : null;
    groups.set(name, [...(groups.get(name) ?? []), item]);
  }
  return [...groups.entries()]
    .map(([name, grouped]) => ({ name, items: grouped }))
    .sort((left, right) => (left.name === null ? 1 : right.name === null ? -1 : left.name.localeCompare(right.name, locale)));
}
