import { z } from "zod";

export const roadmapNodeSchema = z.object({
  id: z.string(),
  title: z.string(),
  type: z.enum(["foundation", "concept", "skill", "practice", "checkpoint", "project"]),
  status: z.enum(["locked", "ready", "active", "completed", "blocked"]),
  coverageStatus: z.enum(["good", "partial", "missing"]),
  lessonId: z.string().optional(),
  position: z
    .object({
      x: z.number(),
      y: z.number(),
    })
    .optional(),
});

export const roadmapEdgeSchema = z.object({
  id: z.string(),
  source: z.string(),
  target: z.string(),
  type: z.enum(["prerequisite", "recommended"]).optional(),
});

export const roadmapArtifactSchema = z.object({
  kind: z.literal("roadmap"),
  id: z.string(),
  title: z.string(),
  goal: z.string(),
  status: z.enum(["draft", "active", "completed", "needs_review"]),
  coverageStatus: z.enum(["good", "partial", "missing"]),
  nodes: z.array(roadmapNodeSchema),
  edges: z.array(roadmapEdgeSchema),
  metadata: z
    .object({
      estimatedWeeks: z.number().optional(),
      difficulty: z.enum(["beginner", "intermediate", "advanced"]).optional(),
      generatedAt: z.string().optional(),
    })
    .catchall(z.unknown())
    .optional(),
});

export const resourceEvidenceSchema = z.object({
  id: z.string(),
  title: z.string(),
  url: z.string().url().optional(),
  sourceType: z.enum(["official_docs", "article", "video", "exercise", "project"]),
  trustTier: z.union([z.literal(1), z.literal(2), z.literal(3)]),
});

export const exerciseSchema = z.object({
  id: z.string(),
  prompt: z.string(),
  expectedOutput: z.string().optional(),
  difficulty: z.enum(["easy", "medium", "hard"]).optional(),
});

export const lessonArtifactSchema = z.object({
  kind: z.literal("lesson"),
  id: z.string(),
  roadmapId: z.string(),
  nodeId: z.string(),
  title: z.string(),
  objective: z.string(),
  explanation: z.string(),
  resources: z.array(resourceEvidenceSchema),
  exercise: exerciseSchema.optional(),
  status: z.enum(["not_started", "active", "completed"]),
});

export const resourceReadinessArtifactSchema = z.object({
  kind: z.literal("resource_readiness"),
  id: z.string(),
  topicId: z.string(),
  topicName: z.string(),
  overallStatus: z.enum(["good", "partial", "low"]),
  officialDocsCoverage: z.number(),
  exercisesCoverage: z.number(),
  videosCoverage: z.number(),
  projectsCoverage: z.number(),
  missingAreas: z.array(z.string()),
  recommendedAction: z.enum(["start_learning", "backfill_first", "review_sources"]).optional(),
});

export const gradeResultArtifactSchema = z.object({
  kind: z.literal("grade_result"),
  id: z.string(),
  lessonId: z.string(),
  exerciseId: z.string().optional(),
  score: z.number(),
  maxScore: z.number(),
  status: z.enum(["pass", "partial", "fail"]),
  feedback: z.string(),
  strengths: z.array(z.string()),
  issues: z.array(z.string()),
  nextAction: z.enum(["retry", "continue", "review_lesson"]),
});

export const backfillJobArtifactSchema = z.object({
  kind: z.literal("backfill_job"),
  id: z.string(),
  topicId: z.string(),
  status: z.enum(["queued", "running", "completed", "failed"]),
  progress: z.number().optional(),
  message: z.string().optional(),
  createdAt: z.string(),
  completedAt: z.string().optional(),
});

export const uiArtifactSchema = z.discriminatedUnion("kind", [
  roadmapArtifactSchema,
  lessonArtifactSchema,
  resourceReadinessArtifactSchema,
  gradeResultArtifactSchema,
  backfillJobArtifactSchema,
]);

export type UIArtifact = z.infer<typeof uiArtifactSchema>;
export type RoadmapArtifact = z.infer<typeof roadmapArtifactSchema>;
