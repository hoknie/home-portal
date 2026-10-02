import { z } from "zod";

export const alignSchema = z.enum(["start", "center", "end"]).catch("start");

export type Align = z.infer<typeof alignSchema>;

export const toneSchema = z.enum(["neutral", "ok", "info", "warning", "danger"]).catch("neutral");

export type Tone = z.infer<typeof toneSchema>;

export const textSizeSchema = z.enum(["small", "normal", "large"]).catch("normal");

export type TextSize = z.infer<typeof textSizeSchema>;

export const weightSchema = z.enum(["normal", "strong"]).catch("normal");

export type Weight = z.infer<typeof weightSchema>;

export const renderedItemSchema = z.object({ "secondary": z.string().nullable(), "text": z.string(), "tone": toneSchema });

export type RenderedItem = z.infer<typeof renderedItemSchema>;

export const renderedPairSchema = z.object({ "key": z.string(), "value": z.string() });

export type RenderedPair = z.infer<typeof renderedPairSchema>;

export const actionKindSchema = z.enum(["automation", "workflow", "refresh", "link"]).catch("refresh");

export type ActionKind = z.infer<typeof actionKindSchema>;

export const buttonStyleSchema = z.enum(["primary", "secondary", "ghost"]).catch("secondary");

export type ButtonStyle = z.infer<typeof buttonStyleSchema>;

export const rowAlignSchema = z.enum(["start", "center", "end", "stretch"]).catch("stretch");

export type RowAlign = z.infer<typeof rowAlignSchema>;

export const renderedLeafSchema = z.union([z.object({ "align": alignSchema.nullable(), "caption": z.string().nullable(), "icon": z.string().nullable(), "kind": z.literal("stat"), "label": z.string(), "tone": toneSchema, "unit": z.string().nullable(), "valign": alignSchema.nullable(), "value": z.string() }), z.object({ "align": alignSchema.nullable(), "kind": z.literal("text"), "muted": z.boolean(), "size": textSizeSchema, "text": z.string(), "tone": toneSchema, "valign": alignSchema.nullable(), "weight": weightSchema }), z.object({ "align": alignSchema.nullable(), "kind": z.literal("markdown"), "text": z.string(), "valign": alignSchema.nullable() }), z.object({ "align": alignSchema.nullable(), "empty": z.string().nullable(), "items": z.array(renderedItemSchema), "kind": z.literal("list"), "more": z.number(), "valign": alignSchema.nullable() }), z.object({ "align": alignSchema.nullable(), "empty": z.string().nullable(), "headers": z.array(z.string()), "kind": z.literal("table"), "more": z.number(), "rows": z.array(z.array(z.string())), "valign": alignSchema.nullable() }), z.object({ "align": alignSchema.nullable(), "kind": z.literal("key-values"), "pairs": z.array(renderedPairSchema), "valign": alignSchema.nullable() }), z.object({ "align": alignSchema.nullable(), "caption": z.string().nullable(), "kind": z.literal("progress"), "label": z.string().nullable(), "maximum": z.number(), "tone": toneSchema, "valign": alignSchema.nullable(), "value": z.number() }), z.object({ "align": alignSchema.nullable(), "kind": z.literal("badge"), "text": z.string(), "tone": toneSchema, "valign": alignSchema.nullable() }), z.object({ "action": z.string(), "align": alignSchema.nullable(), "confirm": z.string().nullable(), "does": actionKindSchema, "icon": z.string().nullable(), "kind": z.literal("button"), "label": z.string(), "link": z.string().nullable(), "style": buttonStyleSchema, "valign": alignSchema.nullable() }), z.object({ "kind": z.literal("divider") })]);

export type RenderedLeaf = z.infer<typeof renderedLeafSchema>;

export const gapSchema = z.enum(["small", "normal"]).catch("normal");

export type Gap = z.infer<typeof gapSchema>;

export const renderedGroupSchema = z.union([z.object({ "align": alignSchema.nullable(), "caption": z.string().nullable(), "icon": z.string().nullable(), "kind": z.literal("stat"), "label": z.string(), "tone": toneSchema, "unit": z.string().nullable(), "valign": alignSchema.nullable(), "value": z.string() }), z.object({ "align": alignSchema.nullable(), "kind": z.literal("text"), "muted": z.boolean(), "size": textSizeSchema, "text": z.string(), "tone": toneSchema, "valign": alignSchema.nullable(), "weight": weightSchema }), z.object({ "align": alignSchema.nullable(), "kind": z.literal("markdown"), "text": z.string(), "valign": alignSchema.nullable() }), z.object({ "align": alignSchema.nullable(), "empty": z.string().nullable(), "items": z.array(renderedItemSchema), "kind": z.literal("list"), "more": z.number(), "valign": alignSchema.nullable() }), z.object({ "align": alignSchema.nullable(), "empty": z.string().nullable(), "headers": z.array(z.string()), "kind": z.literal("table"), "more": z.number(), "rows": z.array(z.array(z.string())), "valign": alignSchema.nullable() }), z.object({ "align": alignSchema.nullable(), "kind": z.literal("key-values"), "pairs": z.array(renderedPairSchema), "valign": alignSchema.nullable() }), z.object({ "align": alignSchema.nullable(), "caption": z.string().nullable(), "kind": z.literal("progress"), "label": z.string().nullable(), "maximum": z.number(), "tone": toneSchema, "valign": alignSchema.nullable(), "value": z.number() }), z.object({ "align": alignSchema.nullable(), "kind": z.literal("badge"), "text": z.string(), "tone": toneSchema, "valign": alignSchema.nullable() }), z.object({ "action": z.string(), "align": alignSchema.nullable(), "confirm": z.string().nullable(), "does": actionKindSchema, "icon": z.string().nullable(), "kind": z.literal("button"), "label": z.string(), "link": z.string().nullable(), "style": buttonStyleSchema, "valign": alignSchema.nullable() }), z.object({ "kind": z.literal("divider") }), z.object({ "align": rowAlignSchema, "blocks": z.array(renderedLeafSchema), "gap": gapSchema, "kind": z.literal("row"), "widths": z.array(z.number()).nullable() }), z.object({ "align": alignSchema.nullable(), "blocks": z.array(renderedLeafSchema), "gap": gapSchema, "kind": z.literal("column"), "valign": alignSchema.nullable() })]);

export type RenderedGroup = z.infer<typeof renderedGroupSchema>;

export const renderedNestedSchema = z.union([z.object({ "align": alignSchema.nullable(), "caption": z.string().nullable(), "icon": z.string().nullable(), "kind": z.literal("stat"), "label": z.string(), "tone": toneSchema, "unit": z.string().nullable(), "valign": alignSchema.nullable(), "value": z.string() }), z.object({ "align": alignSchema.nullable(), "kind": z.literal("text"), "muted": z.boolean(), "size": textSizeSchema, "text": z.string(), "tone": toneSchema, "valign": alignSchema.nullable(), "weight": weightSchema }), z.object({ "align": alignSchema.nullable(), "kind": z.literal("markdown"), "text": z.string(), "valign": alignSchema.nullable() }), z.object({ "align": alignSchema.nullable(), "empty": z.string().nullable(), "items": z.array(renderedItemSchema), "kind": z.literal("list"), "more": z.number(), "valign": alignSchema.nullable() }), z.object({ "align": alignSchema.nullable(), "empty": z.string().nullable(), "headers": z.array(z.string()), "kind": z.literal("table"), "more": z.number(), "rows": z.array(z.array(z.string())), "valign": alignSchema.nullable() }), z.object({ "align": alignSchema.nullable(), "kind": z.literal("key-values"), "pairs": z.array(renderedPairSchema), "valign": alignSchema.nullable() }), z.object({ "align": alignSchema.nullable(), "caption": z.string().nullable(), "kind": z.literal("progress"), "label": z.string().nullable(), "maximum": z.number(), "tone": toneSchema, "valign": alignSchema.nullable(), "value": z.number() }), z.object({ "align": alignSchema.nullable(), "kind": z.literal("badge"), "text": z.string(), "tone": toneSchema, "valign": alignSchema.nullable() }), z.object({ "action": z.string(), "align": alignSchema.nullable(), "confirm": z.string().nullable(), "does": actionKindSchema, "icon": z.string().nullable(), "kind": z.literal("button"), "label": z.string(), "link": z.string().nullable(), "style": buttonStyleSchema, "valign": alignSchema.nullable() }), z.object({ "kind": z.literal("divider") }), z.object({ "align": rowAlignSchema, "blocks": z.array(renderedGroupSchema), "gap": gapSchema, "kind": z.literal("row"), "widths": z.array(z.number()).nullable() }), z.object({ "align": alignSchema.nullable(), "blocks": z.array(renderedGroupSchema), "gap": gapSchema, "kind": z.literal("column"), "valign": alignSchema.nullable() })]);

export type RenderedNested = z.infer<typeof renderedNestedSchema>;

export const renderedBlockSchema = z.union([z.object({ "align": alignSchema.nullable(), "caption": z.string().nullable(), "icon": z.string().nullable(), "kind": z.literal("stat"), "label": z.string(), "tone": toneSchema, "unit": z.string().nullable(), "valign": alignSchema.nullable(), "value": z.string() }), z.object({ "align": alignSchema.nullable(), "kind": z.literal("text"), "muted": z.boolean(), "size": textSizeSchema, "text": z.string(), "tone": toneSchema, "valign": alignSchema.nullable(), "weight": weightSchema }), z.object({ "align": alignSchema.nullable(), "kind": z.literal("markdown"), "text": z.string(), "valign": alignSchema.nullable() }), z.object({ "align": alignSchema.nullable(), "empty": z.string().nullable(), "items": z.array(renderedItemSchema), "kind": z.literal("list"), "more": z.number(), "valign": alignSchema.nullable() }), z.object({ "align": alignSchema.nullable(), "empty": z.string().nullable(), "headers": z.array(z.string()), "kind": z.literal("table"), "more": z.number(), "rows": z.array(z.array(z.string())), "valign": alignSchema.nullable() }), z.object({ "align": alignSchema.nullable(), "kind": z.literal("key-values"), "pairs": z.array(renderedPairSchema), "valign": alignSchema.nullable() }), z.object({ "align": alignSchema.nullable(), "caption": z.string().nullable(), "kind": z.literal("progress"), "label": z.string().nullable(), "maximum": z.number(), "tone": toneSchema, "valign": alignSchema.nullable(), "value": z.number() }), z.object({ "align": alignSchema.nullable(), "kind": z.literal("badge"), "text": z.string(), "tone": toneSchema, "valign": alignSchema.nullable() }), z.object({ "action": z.string(), "align": alignSchema.nullable(), "confirm": z.string().nullable(), "does": actionKindSchema, "icon": z.string().nullable(), "kind": z.literal("button"), "label": z.string(), "link": z.string().nullable(), "style": buttonStyleSchema, "valign": alignSchema.nullable() }), z.object({ "kind": z.literal("divider") }), z.object({ "align": rowAlignSchema, "blocks": z.array(renderedNestedSchema), "gap": gapSchema, "kind": z.literal("row"), "widths": z.array(z.number()).nullable() }), z.object({ "align": alignSchema.nullable(), "blocks": z.array(renderedNestedSchema), "gap": gapSchema, "kind": z.literal("column"), "valign": alignSchema.nullable() })]);

export type RenderedBlock = z.infer<typeof renderedBlockSchema>;

export const previewErrorResponseSchema = z.object({ "field": z.string(), "message": z.string() });

export type PreviewErrorResponse = z.infer<typeof previewErrorResponseSchema>;

export const declaredPathResponseSchema = z.object({ "description": z.string().nullable(), "kind": z.string(), "path": z.string() });

export type DeclaredPathResponse = z.infer<typeof declaredPathResponseSchema>;

export const widgetPreviewResponseSchema = z.object({ "blocks": z.array(renderedBlockSchema), "data": z.unknown(), "errors": z.array(previewErrorResponseSchema), "paths": z.array(declaredPathResponseSchema), "problem": z.string().nullable(), "ran": z.boolean() });

export type WidgetPreviewResponse = z.infer<typeof widgetPreviewResponseSchema>;

export const schema = widgetPreviewResponseSchema;
