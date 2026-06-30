import { z } from "zod";

export const goalSubmittedIntentSchema = z.object({
  type: z.literal("goal.submitted"),
  payload: z.object({
    goal: z.string().trim().min(1),
    level: z.enum(["beginner", "intermediate", "advanced"]).optional(),
    durationWeeks: z.number().int().positive().optional(),
    hoursPerWeek: z.number().positive().optional(),
    preferredStyle: z.array(z.enum(["docs", "video", "project", "exercise"])).optional(),
    constraints: z.array(z.string()).optional(),
  }),
});

export const freeChatSubmittedIntentSchema = z.object({
  type: z.literal("chat.submitted"),
  payload: z.object({
    message: z.string().trim().min(1),
    contextArtifactId: z.string().optional(),
  }),
});

export const roadmapNodeSelectedIntentSchema = z.object({
  type: z.literal("roadmap.node.selected"),
  payload: z.object({
    roadmapId: z.string().min(1),
    nodeId: z.string().min(1),
  }),
});

export const lessonAnswerSubmittedIntentSchema = z.object({
  type: z.literal("lesson.answer.submitted"),
  payload: z.object({
    lessonId: z.string().min(1),
    answer: z.string().trim().min(1),
    exerciseId: z.string().optional(),
  }),
});

export const resourceBackfillRequestedIntentSchema = z.object({
  type: z.literal("resource.backfill.requested"),
  payload: z.object({
    topicId: z.string().min(1),
    reason: z.string().optional(),
    priority: z.enum(["low", "normal", "high"]).optional(),
  }),
});

export const roadmapRegenerateRequestedIntentSchema = z.object({
  type: z.literal("roadmap.regenerate.requested"),
  payload: z.object({
    roadmapId: z.string().min(1),
    reason: z.string().optional(),
    preserveCompletedNodes: z.boolean().optional(),
  }),
});

export const roadmapScheduleUpdateRequestedIntentSchema = z.object({
  type: z.literal("roadmap.schedule_update.requested"),
  payload: z.object({
    roadmapId: z.string().min(1),
    title: z.string().trim().min(1).optional(),
    nextMilestone: z.string().trim().min(1).optional(),
  }),
});

export const roadmapTaskSelectedIntentSchema = z.object({
  type: z.literal("roadmap.task.selected"),
  payload: z.object({
    roadmapId: z.string().min(1),
    phaseId: z.string().min(1).optional(),
    milestoneId: z.string().min(1).optional(),
    taskId: z.string().min(1),
    title: z.string().trim().min(1),
    description: z.string().optional(),
    level: z.enum(["beginner", "intermediate", "advanced"]).optional(),
  }),
});

export const noteReviewRequestedIntentSchema = z.object({
  type: z.literal("note.review.requested"),
  payload: z.object({
    noteId: z.string().min(1).optional(),
    taskId: z.string().min(1).optional(),
    title: z.string().trim().min(1),
    content: z.string().optional(),
    course: z.string().optional(),
    tags: z.array(z.string()).optional(),
  }),
});

export const reviewTaskSelectedIntentSchema = z.object({
  type: z.literal("review.task.selected"),
  payload: z.object({
    taskId: z.string().min(1).optional(),
    concept: z.string().trim().min(1),
    course: z.string().optional(),
    confidence: z.number().min(0).max(1).optional(),
    due: z.string().optional(),
  }),
});

export const userIntentSchema = z.discriminatedUnion("type", [
  goalSubmittedIntentSchema,
  freeChatSubmittedIntentSchema,
  roadmapNodeSelectedIntentSchema,
  lessonAnswerSubmittedIntentSchema,
  resourceBackfillRequestedIntentSchema,
  roadmapRegenerateRequestedIntentSchema,
  roadmapScheduleUpdateRequestedIntentSchema,
  roadmapTaskSelectedIntentSchema,
  noteReviewRequestedIntentSchema,
  reviewTaskSelectedIntentSchema,
]);

export const sendIntentRequestSchema = z.object({
  intent: userIntentSchema,
});

export type UserIntent = z.infer<typeof userIntentSchema>;
export type SendIntentRequest = z.infer<typeof sendIntentRequestSchema>;
