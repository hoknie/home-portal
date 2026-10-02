export type TemplateTexts = (key: TemplateText) => string;

export type TemplateText =
  | "label"
  | "caption"
  | "used"
  | "name"
  | "value"
  | "status"
  | "refresh"
  | "open"
  | "updated";

export type CustomTemplate = { key: "stat" | "progress" | "list" | "table" | "pairs" | "buttons" | "overview"; blocks: (text: TemplateTexts) => Record<string, unknown>[] };

export const CUSTOM_TEMPLATES: CustomTemplate[] = [
  { key: "stat", blocks: (text) => [{ kind: "stat", label: text("label"), value: "{{data.value}}", caption: text("caption") }] },
  {
    key: "progress",
    blocks: (text) => [{ kind: "progress", label: text("used"), value: "{{data.used}}", maximum: "{{data.total}}", thresholds: { warning: 75, danger: 90 } }],
  },
  { key: "list", blocks: () => [{ kind: "list", items: "{{data.items}}", text: "{{item.name}}", secondary: "{{item.value}}" }] },
  {
    key: "table",
    blocks: (text) => [
      {
        kind: "table",
        items: "{{data.rows}}",
        columns: [
          { header: text("name"), value: "{{item.name}}" },
          { header: text("value"), value: "{{item.value}}" },
        ],
      },
    ],
  },
  { key: "pairs", blocks: (text) => [{ kind: "key-values", pairs: [{ key: text("status"), value: "{{data.status}}" }] }] },
  {
    key: "buttons",
    blocks: (text) => [
      {
        kind: "row",
        blocks: [
          { kind: "button", label: text("refresh"), icon: "refresh-cw", action: { refresh: true } },
          { kind: "button", label: text("open"), style: "ghost", icon: "external-link", action: { link: "https://" } },
        ],
      },
    ],
  },
  {
    key: "overview",
    blocks: (text) => [
      {
        kind: "row",
        widths: [7, 5],
        align: "center",
        blocks: [
          { kind: "stat", label: text("label"), value: "{{data.value}}", caption: text("caption") },
          {
            kind: "column",
            gap: "small",
            align: "end",
            blocks: [
              { kind: "badge", text: "{{data.status}}", tones: { ok: "ok", failed: "danger" } },
              { kind: "text", text: `${text("updated")} {{fetched_at}}`, size: "small", muted: true },
            ],
          },
        ],
      },
    ],
  },
];
