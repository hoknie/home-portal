"use client";

import { cn } from "@/shared/lib/cn";

export type TagFilterProps = {
  label: string;
  tags: string[];
  selected: string[];
  onChange: (selected: string[]) => void;
};

export function tagsMatch(tags: string[], selected: string[]) {
  const own = tags.map((tag) => tag.toLowerCase());
  return selected.every((tag) => own.includes(tag.toLowerCase()));
}

export function distinctTags(lists: string[][]) {
  const seen = new Map<string, string>();
  for (const tag of lists.flat()) {
    if (!seen.has(tag.toLowerCase())) {
      seen.set(tag.toLowerCase(), tag);
    }
  }
  return [...seen.values()].sort((left, right) => left.localeCompare(right));
}

export function stillChosen(selected: string[], tags: string[]) {
  const present = tags.map((tag) => tag.toLowerCase());
  return selected.filter((tag) => present.includes(tag.toLowerCase()));
}

export function TagFilter({ label, tags, selected, onChange }: TagFilterProps) {
  if (tags.length === 0) {
    return null;
  }
  const chosen = selected.map((tag) => tag.toLowerCase());
  return (
    <div role="group" aria-label={label} className="flex flex-wrap items-center gap-1.5">
      <span className="mr-1 text-xs text-muted-foreground">{label}</span>
      {tags.map((tag) => {
        const on = chosen.includes(tag.toLowerCase());
        return (
          <button
            key={tag}
            type="button"
            aria-pressed={on}
            onClick={() => onChange(on ? selected.filter((picked) => picked.toLowerCase() !== tag.toLowerCase()) : [...selected, tag])}
            className={cn(
              "rounded-full border px-2.5 py-0.5 text-xs transition-colors",
              on ? "border-primary bg-primary text-primary-foreground" : "border-glass-edge bg-glass-tint hover:bg-accent",
            )}
          >
            {tag}
          </button>
        );
      })}
    </div>
  );
}
