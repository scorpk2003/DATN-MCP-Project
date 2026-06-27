import { z } from "zod";
import { uiActionRequestSchema } from "./actions.js";
import { uiArtifactSchema } from "./artifacts.js";
import { agentRunSchema, agentSessionSchema, chatMessageSchema, timelineEventSchema } from "./session.js";

export const agentSessionStateSchema = z.object({
  session: agentSessionSchema,
  activeRun: agentRunSchema.optional(),
  messages: z.array(chatMessageSchema),
  artifacts: z.array(uiArtifactSchema),
  pendingActions: z.array(uiActionRequestSchema),
  timeline: z.array(timelineEventSchema),
});

export type AgentSessionState = z.infer<typeof agentSessionStateSchema>;
