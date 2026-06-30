import type { RoadmapArtifact, UIArtifact } from "../protocol/index.js";
import { makeId, nowIso } from "../services/id.js";

type CoverageStatus = "good" | "partial" | "missing";

export type NormalizedOutput =
  | {
      type: "artifacts";
      artifacts: UIArtifact[];
      summary?: string;
    }
  | {
      type: "message";
      message: string;
    };

export function normalizeOrchestratorOutput(
  output: unknown,
  goal: string,
  context?: Record<string, unknown>,
): NormalizedOutput {
  const artifacts = [
    findRoadmapLikeArtifact(output, goal),
    findLessonLikeArtifact(output, context),
    findGradeResultLikeArtifact(output, context),
    findResourceReadinessLikeArtifact(output, context),
    findBackfillJobLikeArtifact(output, context),
    findEmptyReviewFallbackLessonArtifact(output, context),
  ].filter((artifact): artifact is UIArtifact => Boolean(artifact));

  if (artifacts.length > 0) {
    const firstArtifact = artifacts[0];
    return {
      type: "artifacts",
      artifacts,
      summary: artifacts.length === 1 && firstArtifact ? `${firstArtifact.kind} artifact generated.` : `${artifacts.length} artifacts generated.`,
    };
  }

  return {
    type: "message",
    message: summarizeUnknownOutput(output),
  };
}

function findLessonLikeArtifact(value: unknown, context?: Record<string, unknown>): UIArtifact | null {
  const candidate = findObject(
    value,
    (item) =>
      item.kind === "lesson" ||
      isRecord(item.lessonDraft) ||
      isRecord(item.lesson_draft) ||
      Array.isArray(item.contentBlocks) ||
      Array.isArray(item.content_blocks) ||
      (typeof item.explanation === "string" && (Array.isArray(item.resources) || Array.isArray(item.sourceReferences))),
    new Set(),
  );
  if (!candidate) {
    return null;
  }

  const draft = recordFrom(candidate.lessonDraft) ?? recordFrom(candidate.lesson_draft) ?? candidate;
  const resources = arrayFrom(draft.resources) ?? arrayFrom(draft.sourceReferences) ?? arrayFrom(draft.source_references) ?? [];
  const exercise = firstRecord(arrayFrom(draft.exercises)) ?? recordFrom(draft.exercise);
  const id = stringFrom(draft.id) ?? stringFrom(draft.lessonId) ?? stringFrom(candidate.lessonId) ?? makeId("artifact_lesson");
  const contextSourceId = contextReviewSourceId(context);
  const roadmapId =
    stringFrom(draft.roadmapId) ??
    stringFrom(candidate.roadmapId) ??
    stringFrom(context?.roadmapId) ??
    (contextSourceId ? `review_${contextSourceId}` : undefined);
  const nodeId =
    stringFrom(draft.nodeId) ??
    stringFrom(candidate.nodeId) ??
    stringFrom(draft.roadmapNodeId) ??
    stringFrom(candidate.roadmapNodeId) ??
    stringFrom(context?.nodeId) ??
    stringFrom(context?.roadmapNodeId) ??
    contextSourceId;
  if (!roadmapId || !nodeId) {
    return null;
  }

  const explanation =
    stringFrom(draft.explanation) ??
    stringFrom(draft.content) ??
    lessonContentBlockSummary(arrayFrom(draft.contentBlocks) ?? arrayFrom(draft.content_blocks)) ??
    summarizeUnknownOutput(draft);

  return {
    kind: "lesson",
    id,
    roadmapId,
    nodeId,
    title: stringFrom(draft.title) ?? stringFrom(draft.topic) ?? "Generated lesson",
    objective: stringFrom(draft.objective) ?? firstString(arrayFrom(draft.objectives)) ?? "Learn the selected concept.",
    explanation,
    resources: resources.slice(0, 10).map((resource, index) => normalizeResourceEvidence(resource, index)),
    exercise: exercise
      ? {
          id: stringFrom(exercise.id) ?? `exercise_${stringFrom(draft.id) ?? "lesson"}_${Date.now()}`,
          prompt: stringFrom(exercise.prompt) ?? stringFrom(exercise.question) ?? "Complete the practice task.",
          expectedOutput: stringFrom(exercise.expectedOutput) ?? stringFrom(exercise.expected_output),
          difficulty: normalizeDifficulty(stringFrom(exercise.difficulty)),
        }
      : undefined,
    status: normalizeLessonStatus(stringFrom(draft.status)),
  };
}

function findEmptyReviewFallbackLessonArtifact(value: unknown, context?: Record<string, unknown>): UIArtifact | null {
  const contextType = stringFrom(context?.type);
  if (contextType !== "review.task.selected" && contextType !== "note.review.requested") {
    return null;
  }
  if (!isCompletedWithEmptyOutput(value)) {
    return null;
  }

  const sourceId = contextReviewSourceId(context);
  const nodeId = stringFrom(context?.nodeId) ?? stringFrom(context?.roadmapNodeId) ?? sourceId;
  if (!sourceId || !nodeId) {
    return null;
  }

  const concept =
    stringFrom(context?.concept) ??
    stringFrom(context?.title) ??
    stringFrom(context?.course) ??
    "Review session";
  const course = stringFrom(context?.course);
  const confidence = numberFrom(context?.confidence);

  return {
    kind: "lesson",
    id: makeId("artifact_review"),
    roadmapId: stringFrom(context?.roadmapId) ?? `review_${sourceId}`,
    nodeId,
    title: `${concept} review`,
    objective: course ? `Review ${concept} from ${course}.` : `Review ${concept}.`,
    explanation:
      "The orchestrator completed the approval gate without returning generated lesson content, so this review shell keeps the session actionable while the review plan is regenerated.",
    resources: [],
    exercise: {
      id: `exercise_${sourceId}`,
      prompt:
        typeof confidence === "number" && confidence < 0.5
          ? `Explain ${concept} from first principles, then list one point you are still unsure about.`
          : `Summarize ${concept} and write one example from memory.`,
      difficulty: confidence !== undefined && confidence < 0.5 ? "easy" : "medium",
    },
    status: "active",
  };
}

function findGradeResultLikeArtifact(value: unknown, context?: Record<string, unknown>): UIArtifact | null {
  const candidate = findObject(
    value,
    (item) =>
      item.kind === "grade_result" ||
      isRecord(item.gradeResult) ||
      isRecord(item.grade_result) ||
      (typeof item.feedback === "string" && (typeof item.score === "number" || typeof item.score === "string")),
    new Set(),
  );
  if (!candidate) {
    return null;
  }

  const result = recordFrom(candidate.gradeResult) ?? recordFrom(candidate.grade_result) ?? candidate;
  const score = numberFrom(result.score) ?? 0;
  const maxScore = numberFrom(result.maxScore) ?? numberFrom(result.max_score) ?? 100;
  const lessonId = stringFrom(result.lessonId) ?? stringFrom(result.lesson_id) ?? stringFrom(context?.lessonId);
  if (!lessonId) {
    return null;
  }
  return {
    kind: "grade_result",
    id: stringFrom(result.id) ?? makeId("artifact_grade"),
    lessonId,
    exerciseId: stringFrom(result.exerciseId) ?? stringFrom(result.exercise_id),
    score,
    maxScore,
    status: normalizeGradeStatus(stringFrom(result.status), score, maxScore),
    feedback: stringFrom(result.feedback) ?? "Answer graded.",
    strengths: stringArrayFrom(result.strengths),
    issues: stringArrayFrom(result.issues),
    nextAction: normalizeNextAction(stringFrom(result.nextAction) ?? stringFrom(result.next_action)),
  };
}

function findResourceReadinessLikeArtifact(value: unknown, context?: Record<string, unknown>): UIArtifact | null {
  const candidate = findObject(
    value,
    (item) =>
      item.kind === "resource_readiness" ||
      isRecord(item.coverage) ||
      isRecord(item.resourceReadiness) ||
      typeof item.overallStatus === "string" ||
      typeof item.coverageStatus === "string",
    new Set(),
  );
  if (!candidate) {
    return null;
  }

  const readiness = recordFrom(candidate.resourceReadiness) ?? recordFrom(candidate.coverage) ?? candidate;
  const topicName = stringFrom(readiness.topicName) ?? stringFrom(readiness.topic) ?? stringFrom(readiness.topic_text) ?? "Current topic";
  const topicId = stringFrom(readiness.topicId) ?? stringFrom(readiness.topic_id) ?? stringFrom(context?.topicId) ?? slugFrom(topicName);
  return {
    kind: "resource_readiness",
    id: stringFrom(readiness.id) ?? makeId("artifact_readiness"),
    topicId,
    topicName,
    overallStatus: normalizeReadinessStatus(stringFrom(readiness.overallStatus) ?? stringFrom(readiness.coverageStatus)),
    officialDocsCoverage: clamp01(numberFrom(readiness.officialDocsCoverage) ?? numberFrom(readiness.official_docs_coverage) ?? 0),
    exercisesCoverage: clamp01(numberFrom(readiness.exercisesCoverage) ?? numberFrom(readiness.exercises_coverage) ?? 0),
    videosCoverage: clamp01(numberFrom(readiness.videosCoverage) ?? numberFrom(readiness.videos_coverage) ?? 0),
    projectsCoverage: clamp01(numberFrom(readiness.projectsCoverage) ?? numberFrom(readiness.projects_coverage) ?? 0),
    missingAreas: stringArrayFrom(readiness.missingAreas ?? readiness.missing_areas),
    recommendedAction: normalizeRecommendedAction(stringFrom(readiness.recommendedAction) ?? stringFrom(readiness.recommended_action)),
  };
}

function findBackfillJobLikeArtifact(value: unknown, context?: Record<string, unknown>): UIArtifact | null {
  const candidate = findObject(
    value,
    (item) =>
      item.kind === "backfill_job" ||
      isRecord(item.backfillJob) ||
      isRecord(item.researchTask) ||
      isRecord(item.research_task) ||
      typeof item.gapId === "string",
    new Set(),
  );
  if (!candidate) {
    return null;
  }

  const job = recordFrom(candidate.backfillJob) ?? recordFrom(candidate.researchTask) ?? recordFrom(candidate.research_task) ?? candidate;
  const topicId = stringFrom(job.topicId) ?? stringFrom(job.topic_id) ?? stringFrom(job.normalized_query) ?? stringFrom(context?.topicId);
  if (!topicId) {
    return null;
  }
  return {
    kind: "backfill_job",
    id: stringFrom(job.id) ?? stringFrom(job.taskId) ?? stringFrom(job.task_id) ?? makeId("artifact_backfill"),
    topicId,
    status: normalizeBackfillStatus(stringFrom(job.status)),
    progress: clamp01(numberFrom(job.progress) ?? 0),
    message: stringFrom(job.message) ?? stringFrom(job.description),
    createdAt: stringFrom(job.createdAt) ?? stringFrom(job.created_at) ?? nowIso(),
    completedAt: stringFrom(job.completedAt) ?? stringFrom(job.completed_at),
  };
}

function findRoadmapLikeArtifact(value: unknown, goal: string): RoadmapArtifact | null {
  const seen = new Set<unknown>();
  const candidate = findObject(value, (item) => Array.isArray(item.nodes) || Array.isArray(item.phases) || Array.isArray(item.steps), seen);
  if (!candidate) {
    return null;
  }
  const roadmapId = stringFrom(candidate.id) ?? stringFrom(candidate.roadmapId) ?? stringFrom(candidate.roadmap_id);
  if (!roadmapId) {
    return null;
  }

  const rawNodes = arrayFrom(candidate.nodes) ?? arrayFrom(candidate.phases) ?? arrayFrom(candidate.steps) ?? [];
  const nodes = rawNodes
    .slice(0, 40)
    .map((node, index) => {
      const record = isRecord(node) ? node : {};
      const id = stringFrom(record.id) ?? stringFrom(record.nodeId) ?? stringFrom(record.node_id) ?? stringFrom(record.slug);
      if (!id) {
        return null;
      }
      const title =
        stringFrom(record.title) ??
        stringFrom(record.name) ??
        stringFrom(record.label) ??
        stringFrom(record.topic) ??
        `Learning step ${index + 1}`;

      return {
        id,
        title,
        type: normalizeNodeType(stringFrom(record.type) ?? stringFrom(record.nodeType) ?? stringFrom(record.node_type)),
        status: normalizeNodeStatus(stringFrom(record.status), index),
        coverageStatus: normalizeCoverage(stringFrom(record.coverageStatus) ?? stringFrom(record.coverage)),
        lessonId: stringFrom(record.lessonId) ?? stringFrom(record.lesson_id),
        position: {
          x: 120 + (index % 4) * 220,
          y: 100 + Math.floor(index / 4) * 160,
        },
      };
    })
    .filter((node): node is NonNullable<typeof node> => Boolean(node));

  if (nodes.length === 0) {
    return null;
  }

  const rawEdges = arrayFrom(candidate.edges);
  const nodeIds = new Set(nodes.map((node) => node.id));
  const edges = rawEdges
    ? rawEdges
        .map((edge, index) => {
          const record = isRecord(edge) ? edge : {};
          const source =
            stringFrom(record.source) ??
            stringFrom(record.from) ??
            stringFrom(record.fromNodeId) ??
            stringFrom(record.from_node_id);
          const target =
            stringFrom(record.target) ??
            stringFrom(record.to) ??
            stringFrom(record.toNodeId) ??
            stringFrom(record.to_node_id);
          if (!source || !target || !nodeIds.has(source) || !nodeIds.has(target)) {
            return null;
          }
          return {
            id: stringFrom(record.id) ?? `edge_${source}_${target}_${index + 1}`,
            source,
            target,
            type: normalizeEdgeType(stringFrom(record.type) ?? stringFrom(record.edgeType) ?? stringFrom(record.edge_type)),
          };
        })
        .filter((edge): edge is NonNullable<typeof edge> => Boolean(edge))
    : nodes.slice(1).map((node, index) => ({
        id: `edge_${nodes[index]?.id}_${node.id}`,
        source: nodes[index]?.id ?? nodes[0]?.id ?? node.id,
        target: node.id,
        type: "recommended" as const,
      }));

  return {
    kind: "roadmap",
    id: roadmapId,
    title: stringFrom(candidate.title) ?? "Generated learning roadmap",
    goal: stringFrom(candidate.goal) ?? goal,
    status: "draft",
    coverageStatus: normalizeCoverage(
      stringFrom(candidate.coverageStatus) ?? stringFrom(recordFrom(candidate.coverageSummary)?.overallStatus),
    ),
    nodes,
    edges,
    metadata: {
      generatedAt: nowIso(),
    },
  };
}

function findObject(
  value: unknown,
  predicate: (item: Record<string, unknown>) => boolean,
  seen: Set<unknown>,
): Record<string, unknown> | null {
  if (!isRecord(value) || seen.has(value)) {
    return null;
  }
  seen.add(value);

  if (predicate(value)) {
    return value;
  }

  for (const child of Object.values(value)) {
    if (Array.isArray(child)) {
      for (const item of child) {
        const found = findObject(item, predicate, seen);
        if (found) {
          return found;
        }
      }
    } else {
      const found = findObject(child, predicate, seen);
      if (found) {
        return found;
      }
    }
  }

  return null;
}

function summarizeUnknownOutput(output: unknown) {
  if (typeof output === "string" && output.trim()) {
    return output.trim();
  }

  if (isRecord(output)) {
    const message = stringFrom(output.message) ?? stringFrom(output.summary) ?? stringFrom(output.answer);
    if (message) {
      return message;
    }
  }

  return "Agent completed the run, but the result did not match a supported UI artifact yet.";
}

function normalizeNodeType(value?: string) {
  const allowed = ["foundation", "concept", "skill", "practice", "checkpoint", "project"] as const;
  return allowed.find((item) => item === value) ?? "concept";
}

function normalizeNodeStatus(value: string | undefined, index: number) {
  const allowed = ["locked", "ready", "active", "completed", "blocked"] as const;
  return allowed.find((item) => item === value) ?? (index === 0 ? "active" : "ready");
}

function normalizeCoverage(value?: string): CoverageStatus {
  if (value === "missing" || value === "low" || value === "poor") {
    return "missing";
  }
  if (value === "partial") {
    return "partial";
  }
  return "good";
}

function normalizeResourceEvidence(value: unknown, index: number) {
  const record: Record<string, unknown> = isRecord(value) ? value : {};
  return {
    id: stringFrom(record.id) ?? stringFrom(record.resourceId) ?? stringFrom(record.resource_id) ?? `resource_${index + 1}`,
    title: stringFrom(record.title) ?? "Learning resource",
    url: stringFrom(record.url) ?? stringFrom(record.canonicalUrl) ?? stringFrom(record.canonical_url),
    sourceType: normalizeSourceType(stringFrom(record.sourceType) ?? stringFrom(record.source_type) ?? stringFrom(record.kind)),
    trustTier: normalizeTrustTier(numberFrom(record.trustTier) ?? numberFrom(record.trust_tier)),
  };
}

function normalizeSourceType(value?: string): "official_docs" | "article" | "video" | "exercise" | "project" {
  if (value === "official_docs" || value === "video" || value === "exercise" || value === "project") {
    return value;
  }
  if (value === "docs") {
    return "official_docs";
  }
  return "article";
}

function normalizeTrustTier(value?: number): 1 | 2 | 3 {
  if (value === 1 || value === 2 || value === 3) {
    return value;
  }
  return 3;
}

function normalizeDifficulty(value?: string): "easy" | "medium" | "hard" | undefined {
  return value === "easy" || value === "medium" || value === "hard" ? value : undefined;
}

function normalizeLessonStatus(value?: string): "not_started" | "active" | "completed" {
  if (value === "completed") {
    return "completed";
  }
  if (value === "active" || value === "ready") {
    return "active";
  }
  return "not_started";
}

function normalizeGradeStatus(value: string | undefined, score: number, maxScore: number): "pass" | "partial" | "fail" {
  if (value === "pass" || value === "partial" || value === "fail") {
    return value;
  }
  const ratio = maxScore > 0 ? score / maxScore : 0;
  if (ratio >= 0.8) {
    return "pass";
  }
  if (ratio >= 0.45) {
    return "partial";
  }
  return "fail";
}

function normalizeNextAction(value?: string): "retry" | "continue" | "review_lesson" {
  if (value === "retry" || value === "continue" || value === "review_lesson") {
    return value;
  }
  return "continue";
}

function normalizeReadinessStatus(value?: string): "good" | "partial" | "low" {
  if (value === "good" || value === "partial" || value === "low") {
    return value;
  }
  if (value === "missing") {
    return "low";
  }
  return "partial";
}

function normalizeRecommendedAction(value?: string): "start_learning" | "backfill_first" | "review_sources" | undefined {
  if (value === "start_learning" || value === "backfill_first" || value === "review_sources") {
    return value;
  }
  return undefined;
}

function normalizeBackfillStatus(value?: string): "queued" | "running" | "completed" | "failed" {
  if (value === "running" || value === "completed" || value === "failed") {
    return value;
  }
  return "queued";
}

function normalizeEdgeType(value?: string): "prerequisite" | "recommended" {
  return value === "recommended" || value === "recommended_before" || value === "optional_before" ? "recommended" : "prerequisite";
}

function arrayFrom(value: unknown) {
  return Array.isArray(value) ? value : null;
}

function firstRecord(value: unknown[] | null) {
  return value?.find(isRecord);
}

function recordFrom(value: unknown) {
  return isRecord(value) ? value : null;
}

function stringFrom(value: unknown) {
  return typeof value === "string" && value.trim() ? value.trim() : undefined;
}

function firstString(value: unknown[] | null) {
  return value?.find((item): item is string => typeof item === "string" && item.trim().length > 0);
}

function stringArrayFrom(value: unknown) {
  return Array.isArray(value) ? value.filter((item): item is string => typeof item === "string" && item.trim().length > 0) : [];
}

function numberFrom(value: unknown) {
  if (typeof value === "number" && Number.isFinite(value)) {
    return value;
  }
  if (typeof value === "string" && value.trim()) {
    const parsed = Number(value);
    return Number.isFinite(parsed) ? parsed : undefined;
  }
  return undefined;
}

function clamp01(value: number) {
  return Math.max(0, Math.min(1, value));
}

function slugFrom(value: string) {
  return value.toLowerCase().replace(/[^a-z0-9]+/g, "-").replace(/^-|-$/g, "") || "current-topic";
}

function contextReviewSourceId(context?: Record<string, unknown>) {
  const source = (
    stringFrom(context?.taskId) ??
    stringFrom(context?.noteId) ??
    stringFrom(context?.concept) ??
    stringFrom(context?.title) ??
    stringFrom(context?.course)
  )?.replace(/[^a-zA-Z0-9_-]+/g, "-").replace(/^-|-$/g, "");
  return source || undefined;
}

function lessonContentBlockSummary(blocks: unknown[] | null) {
  const block = blocks?.find(isRecord);
  if (!block) {
    return undefined;
  }
  return stringFrom(block.content) ?? stringFrom(block.summary) ?? stringFrom(block.title);
}

function isCompletedWithEmptyOutput(value: unknown) {
  const record = recordFrom(value);
  if (!record) {
    return false;
  }
  const status = stringFrom(record.status);
  if (status !== "completed") {
    return false;
  }
  const output = record.output;
  if (output === undefined || output === null) {
    return true;
  }
  if (isRecord(output)) {
    return Object.keys(output).length === 0;
  }
  if (Array.isArray(output)) {
    return output.length === 0;
  }
  return false;
}

function isRecord(value: unknown): value is Record<string, unknown> {
  return typeof value === "object" && value !== null && !Array.isArray(value);
}
