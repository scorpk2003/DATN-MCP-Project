import { z } from "zod";

export const agentSessionSchema = z.object({
  id: z.string(),
  userId: z.string().optional(),
  title: z.string().optional(),
  status: z.enum(["active", "archived"]),
  createdAt: z.string(),
  updatedAt: z.string(),
  metadata: z.record(z.string(), z.unknown()).optional(),
});

export const agentRunSchema = z.object({
  id: z.string(),
  sessionId: z.string(),
  parentRunId: z.string().optional(),
  status: z.enum(["queued", "running", "waiting_for_user", "completed", "failed", "cancelled"]),
  currentStep: z.string().optional(),
  startedAt: z.string(),
  completedAt: z.string().optional(),
  error: z.string().optional(),
  metadata: z.record(z.string(), z.unknown()).optional(),
});

export const chatMessageSchema = z.object({
  id: z.string(),
  role: z.enum(["user", "agent", "system"]),
  content: z.string(),
  runId: z.string().optional(),
  createdAt: z.string(),
});

export const timelineEventSchema = z.object({
  id: z.string(),
  runId: z.string(),
  type: z.string(),
  label: z.string(),
  status: z.enum(["started", "completed", "failed"]).optional(),
  createdAt: z.string(),
});

export const createSessionRequestSchema = z.object({
  userId: z.string().optional(),
  title: z.string().optional(),
  metadata: z
    .object({
      source: z.enum(["web", "mobile", "ide"]).optional(),
      locale: z.string().optional(),
    })
    .catchall(z.unknown())
    .optional(),
});

export type AgentSession = z.infer<typeof agentSessionSchema>;
export type AgentRun = z.infer<typeof agentRunSchema>;
export type ChatMessage = z.infer<typeof chatMessageSchema>;
export type AgentTimelineEvent = z.infer<typeof timelineEventSchema>;
export type CreateSessionRequest = z.infer<typeof createSessionRequestSchema>;
