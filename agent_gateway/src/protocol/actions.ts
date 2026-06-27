import { z } from "zod";

export const uiActionOptionSchema = z.object({
  id: z.string(),
  label: z.string(),
  variant: z.enum(["primary", "secondary", "danger"]).optional(),
});

export const uiActionRequestSchema = z.object({
  id: z.string(),
  runId: z.string(),
  title: z.string(),
  description: z.string().optional(),
  options: z.array(uiActionOptionSchema).min(1),
  status: z.enum(["pending", "resolved"]).default("pending"),
  createdAt: z.string(),
  resolvedAt: z.string().optional(),
});

export const respondActionRequestSchema = z.object({
  selectedOptionId: z.string().min(1),
  payload: z.record(z.string(), z.unknown()).optional(),
});

export type UIActionRequest = z.infer<typeof uiActionRequestSchema>;
export type RespondActionRequest = z.infer<typeof respondActionRequestSchema>;
