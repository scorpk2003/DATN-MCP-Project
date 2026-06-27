import { z } from "zod";
import { uiActionRequestSchema } from "./actions.js";
import { uiArtifactSchema } from "./artifacts.js";
import { agentRunSchema } from "./session.js";

export const agentEventSchema = z.discriminatedUnion("type", [
  z.object({
    type: z.literal("agent.message"),
    message: z.string(),
  }),
  z.object({
    type: z.literal("agent.thinking"),
    label: z.string(),
  }),
  z.object({
    type: z.literal("tool.started"),
    toolName: z.string(),
    displayName: z.string(),
  }),
  z.object({
    type: z.literal("tool.completed"),
    toolName: z.string(),
    displayName: z.string(),
    resultSummary: z.string().optional(),
  }),
  z.object({
    type: z.literal("artifact.created"),
    artifact: uiArtifactSchema,
  }),
  z.object({
    type: z.literal("artifact.updated"),
    artifactId: z.string(),
    patch: z.record(z.string(), z.unknown()),
  }),
  z.object({
    type: z.literal("ui.action_required"),
    action: uiActionRequestSchema,
  }),
  z.object({
    type: z.literal("run.status_changed"),
    status: agentRunSchema.shape.status,
    currentStep: z.string().optional(),
  }),
  z.object({
    type: z.literal("error"),
    message: z.string(),
    recoverable: z.boolean(),
    code: z.string().optional(),
  }),
]);

export const agentStreamEnvelopeSchema = z.object({
  id: z.string(),
  sessionId: z.string(),
  runId: z.string(),
  sequence: z.number().int().positive(),
  timestamp: z.string(),
  event: agentEventSchema,
});

export type AgentEvent = z.infer<typeof agentEventSchema>;
export type AgentStreamEnvelope = z.infer<typeof agentStreamEnvelopeSchema>;
