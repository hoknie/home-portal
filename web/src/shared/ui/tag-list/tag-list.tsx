import { Badge } from "@/shared/ui/primitives";

export function TagList({ tags }: { tags: string[] }) {
  if (tags.length === 0) {
    return null;
  }
  return (
    <span className="flex flex-wrap gap-1">
      {tags.map((tag) => (
        <Badge key={tag} variant="outline" className="text-[11px]">
          {tag}
        </Badge>
      ))}
    </span>
  );
}
