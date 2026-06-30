import { Router } from "express";
import type { GatewayConfig } from "../config.js";
import { asyncHandler } from "../middleware/asyncHandler.js";
import type { RoadmapArtifact } from "../protocol/index.js";
import { buildAuthContext } from "../services/authContext.js";
import { sessionStore } from "../services/sessionStore.js";

const learner = {
  name: "Khang",
  focus: "AI Engineer roadmap",
  streakDays: 12,
  weeklyGoalHours: 18,
  completedHours: 11,
};

const recentChats = [
  { id: "chat-01", label: "Lập kế hoạch học RAG" },
  { id: "chat-02", label: "Ôn lại vector database" },
  { id: "chat-03", label: "Giải thích backpropagation" },
  { id: "chat-04", label: "Checklist đồ án tốt nghiệp" },
];

const sidebarItems = [
  { id: "home", label: "Tổng quan", path: "/" },
  { id: "roadmap", label: "Lộ trình", path: "/roadmap" },
  { id: "review", label: "Ôn tập", path: "/review" },
  { id: "notes", label: "Ghi chú", path: "/notes" },
  { id: "resources", label: "Tài liệu", path: "/resources" },
];

const quickPrompts = [
  "Tạo roadmap 30 ngày",
  "Tóm tắt bài vừa học",
  "Sinh câu hỏi ôn tập",
  "Gợi ý tài liệu tiếp theo",
];

const learningMetrics = [
  { id: "progress", label: "Tiến độ tuần", value: "61%", tone: "success" },
  { id: "focus", label: "Phiên tập trung", value: "8", tone: "info" },
  { id: "review", label: "Cần ôn tập", value: "14", tone: "warning" },
];

const recommendedCourses = [
  {
    id: "course-rag",
    title: "RAG Foundation",
    description: "Hiểu pipeline truy xuất, chunking, embedding và đánh giá câu trả lời.",
    progress: 0.72,
    status: "in-progress",
    statusLabel: "Đang học",
    duration: "6 bài",
    tone: "info",
  },
  {
    id: "course-agent",
    title: "AI Agent Workflow",
    description: "Thiết kế planner, tool calling, memory và quan sát luồng tác vụ.",
    progress: 0.34,
    status: "in-progress",
    statusLabel: "Tiếp tục",
    duration: "9 bài",
    tone: "agent",
  },
  {
    id: "course-mlops",
    title: "MLOps Starter",
    description: "Chuẩn hóa experiment, tracking, evaluation và triển khai model nhỏ.",
    progress: 0.12,
    status: "not-started",
    statusLabel: "Mới",
    duration: "5 bài",
    tone: "teal",
  },
];

const agentActivities = [
  {
    id: "activity-01",
    title: "Đã tạo nhánh học hôm nay",
    description: "Ưu tiên embedding evaluation trước khi chuyển sang reranking.",
    tone: "success",
  },
  {
    id: "activity-02",
    title: "Phát hiện lỗ hổng kiến thức",
    description: "Bạn cần ôn lại cosine similarity và recall@k.",
    tone: "warning",
  },
  {
    id: "activity-03",
    title: "Tài liệu phù hợp",
    description: "Gợi ý đọc thêm về query decomposition cho đồ án.",
    tone: "info",
  },
];

const resourcesFallback = [
  {
    id: "resource-rag-checklist",
    title: "RAG evaluation checklist",
    description: "Checklist đánh giá retrieval, grounding và answer quality cho đồ án.",
    type: "Checklist",
    source: "Mentor agent",
    course: "RAG Foundation",
    saved: true,
    tone: "info",
    updatedAt: "2026-06-26",
  },
  {
    id: "resource-embedding-notes",
    title: "Embedding search notes",
    description: "Tóm tắt cách chọn embedding model, index và metric phù hợp.",
    type: "Note",
    source: "Saved from chat",
    course: "RAG Foundation",
    saved: true,
    tone: "teal",
    updatedAt: "2026-06-25",
  },
  {
    id: "resource-agent-patterns",
    title: "Agent tool calling patterns",
    description: "Các pattern chia tool, schema và retry loop cho agent workflow.",
    type: "Guide",
    source: "Course material",
    course: "AI Agent Workflow",
    saved: false,
    tone: "agent",
    updatedAt: "2026-06-24",
  },
];

const notes = [
  {
    id: "note-cosine",
    title: "Cosine similarity",
    excerpt: "Đo độ gần nhau giữa hai vector bằng góc, hữu ích khi so sánh embedding.",
    content:
      "Cosine similarity tập trung vào hướng của vector thay vì độ lớn. Với RAG, metric này thường dùng để xếp hạng document chunk theo độ gần ngữ nghĩa với query.",
    course: "RAG Foundation",
    tags: ["retrieval", "math"],
    updatedAt: "2026-06-26",
    pinned: true,
    tone: "warning",
  },
  {
    id: "note-recall-k",
    title: "Recall@k",
    excerpt: "Tỷ lệ item liên quan xuất hiện trong top-k kết quả truy xuất.",
    content:
      "Recall@k giúp đánh giá retrieval trước khi đánh giá generation. Nếu recall thấp, câu trả lời cuối cùng khó đúng dù prompt hoặc model tốt.",
    course: "RAG Foundation",
    tags: ["evaluation", "retrieval"],
    updatedAt: "2026-06-25",
    pinned: false,
    tone: "info",
  },
];

const reviewQueue = [
  {
    id: "review-01",
    concept: "Cosine similarity",
    course: "RAG Foundation",
    due: "Hôm nay",
    confidence: 0.58,
    tone: "warning",
  },
  {
    id: "review-02",
    concept: "Recall@k",
    course: "RAG Foundation",
    due: "Hôm nay",
    confidence: 0.42,
    tone: "risk",
  },
];

const roadmapSummary = {
  title: "AI Engineer trong 12 tuần",
  description:
    "Lộ trình ưu tiên nền tảng ML, RAG, agent workflow và MLOps nhẹ để phục vụ đồ án self-learning.",
  progress: 0.46,
  currentSprint: "Tuần 5",
  nextMilestone: "Hoàn thiện RAG evaluation",
};

const roadmapPhases = [
  {
    id: "foundation",
    title: "Nền tảng ML",
    duration: "Tuần 1-3",
    status: "completed",
    statusLabel: "Hoàn tất",
    tone: "success",
    progress: 1,
    tasks: ["Linear models", "Neural network basics", "Evaluation metrics"],
  },
  {
    id: "retrieval",
    title: "Retrieval & RAG",
    duration: "Tuần 4-6",
    status: "in-progress",
    statusLabel: "Đang học",
    tone: "info",
    progress: 0.62,
    tasks: ["Chunking strategy", "Embedding search", "Answer evaluation"],
  },
];

export function dataRouter(config: GatewayConfig) {
  const router = Router();

  router.get(
    "/dashboard",
    asyncHandler(async (_request, response) => {
      response.json({
        learner,
        quickPrompts,
        learningMetrics,
        recommendedCourses,
        agentActivities,
      });
    }),
  );

  router.get(
    "/courses/recommended",
    asyncHandler(async (_request, response) => {
      response.json(recommendedCourses);
    }),
  );

  router.get(
    "/navigation",
    asyncHandler(async (request, response) => {
      const authContext = buildAuthContext(request, config);
      const scopedSessions = authContext?.userId
        ? sessionStore.listSessions({ userId: authContext.userId, limit: 10 })
        : [];
      response.json({
        recentChats: scopedSessions.map((session) => ({
          id: session.id,
          label: session.title || session.id,
          path: "/",
          updatedAt: session.updatedAt,
        })),
        sidebarItems,
      });
    }),
  );

  router.get(
    "/resources",
    asyncHandler(async (_request, response) => {
      const resources: GatewayResource[] = await readResourcesFromResourceService(config).catch(() => []);
      response.json({
        resources,
        resourceCourses: uniqueLabels(["Tất cả", ...resources.map((resource) => resource.course)]),
        resourceTypes: uniqueLabels(["Tất cả", ...resources.map((resource) => resource.type)]),
      });
    }),
  );

  router.get(
    "/notes",
    asyncHandler(async (request, response) => {
      const authContext = buildAuthContext(request, config);
      const persistedNotes = authContext?.userId ? await readPersistedNotes(config, authContext.userId) : [];
      response.json({
        notes: persistedNotes.map(persistedNoteToPageData),
        noteCourses: uniqueLabels(["Tất cả", ...persistedNotes.map((note) => readString(note.project_title) ?? "General")]),
      });
    }),
  );

  router.post(
    "/notes",
    asyncHandler(async (request, response) => {
      const authContext = buildAuthContext(request, config);
      if (!authContext?.userId) {
        response.status(401).json({ error: { code: "UNAUTHENTICATED", message: "Authentication is required." } });
        return;
      }

      const content = typeof request.body?.content === "string" ? request.body.content.trim() : "";
      if (!content) {
        response.status(400).json({ error: { code: "INVALID_NOTE", message: "Note content is required." } });
        return;
      }

      const created = await createPersistedNote(config, {
        userId: authContext.userId,
        content,
        taskId: typeof request.body?.taskId === "string" ? request.body.taskId : undefined,
      });
      response.status(201).json({ note: persistedNoteToPageData(created) });
    }),
  );

  router.get(
    "/review",
    asyncHandler(async (request, response) => {
      const authContext = buildAuthContext(request, config);
      const reviewItems = authContext?.userId ? await readPersistedReviewItems(config, authContext.userId) : [];
      response.json({ reviewQueue: reviewItems.map(persistedReviewItemToPageData) });
    }),
  );

  router.get(
    "/roadmap",
    asyncHandler(async (request, response) => {
      const authContext = buildAuthContext(request, config);
      if (!authContext?.userId) {
        response.json(roadmapArtifactToPageData(null));
        return;
      }
      const persistedRoadmap = await readLatestPersistedRoadmap(config, authContext.userId).catch(() => null);
      response.json(
        persistedRoadmap
          ? persistedRoadmapToPageData(persistedRoadmap)
          : roadmapArtifactToPageData(sessionStore.getLatestRoadmapArtifact({ userId: authContext.userId })),
      );
    }),
  );

  return router;
}

type GatewayResource = {
  id: string;
  title: string;
  description: string;
  type: string;
  source: string;
  course: string;
  saved: boolean;
  tone: string;
  updatedAt: string;
  url?: string;
};

async function readResourcesFromResourceService(config: GatewayConfig): Promise<GatewayResource[]> {
  const url = `${config.resourceServiceBaseUrl.replace(/\/$/, "")}/resources`;
  const response = await fetch(url);
  if (!response.ok) {
    throw new Error(`Resource Service returned ${response.status}`);
  }

  const body = await response.json();
  const items = readResourceItems(body);

  return items.slice(0, 100).map((item: Record<string, unknown>, index: number) => ({
    id: readString(item.id) ?? readString(item.resourceId) ?? `resource-${index + 1}`,
    title: readString(item.title) ?? "Untitled resource",
    description: readString(item.description) ?? readString(item.summary) ?? "No description available.",
    type: normalizeResourceType(readString(item.type) ?? readString(item.kind) ?? readString(item.resourceType)),
    source: readString(item.source) ?? readString(item.publisher) ?? readString(item.primary_domain) ?? "Resource Service",
    course: readString(item.course) ?? readString(item.topic) ?? "General",
    saved: Boolean(item.saved),
    tone: "info",
    updatedAt: readString(item.updatedAt) ?? readString(item.updated_at) ?? readString(item.createdAt) ?? readString(item.created_at) ?? new Date().toISOString().slice(0, 10),
    url: readString(item.url) ?? readString(item.canonicalUrl) ?? readString(item.canonical_url),
  }));
}

function readResourceItems(body: unknown): Record<string, unknown>[] {
  if (Array.isArray(body)) {
    return body.filter(isRecord);
  }
  if (!isRecord(body)) {
    return [];
  }

  if (Array.isArray(body.resources)) {
    return body.resources.filter(isRecord);
  }
  if (Array.isArray(body.items)) {
    return body.items.filter(isRecord);
  }
  if (isRecord(body.data)) {
    if (Array.isArray(body.data.items)) {
      return body.data.items.filter(isRecord);
    }
    if (Array.isArray(body.data.resources)) {
      return body.data.resources.filter(isRecord);
    }
  }

  return [];
}

function roadmapArtifactToPageData(artifact: RoadmapArtifact | null) {
  if (!artifact) {
    return {
      roadmapSummary: null,
      roadmapPhases: [],
      studyResources: [],
    };
  }

  const completedNodes = artifact.nodes.filter((node) => node.status === "completed").length;
  const totalNodes = artifact.nodes.length || 1;
  const activeNode = artifact.nodes.find((node) => node.status === "active" || node.status === "ready") ?? artifact.nodes[0];

  return {
    roadmapSummary: {
      id: artifact.id,
      roadmapId: artifact.id,
      title: artifact.title,
      description: artifact.goal,
      progress: completedNodes / totalNodes,
      currentSprint: artifact.status,
      nextMilestone: activeNode?.title ?? "No milestone selected",
    },
    roadmapPhases: artifact.nodes.map((node, index) => ({
      id: node.id,
      roadmapId: artifact.id,
      title: node.title,
      duration: `Step ${index + 1}`,
      status: node.status === "completed" ? "completed" : node.status === "active" || node.status === "ready" ? "in-progress" : "not-started",
      statusLabel: node.status,
      tone: node.coverageStatus === "good" ? "success" : node.coverageStatus === "missing" ? "risk" : "warning",
      progress: node.status === "completed" ? 1 : node.status === "active" ? 0.5 : 0,
      tasks: [
        {
          id: node.id,
          taskId: node.id,
          roadmapId: artifact.id,
          title: node.title,
          type: node.type,
          coverageStatus: node.coverageStatus,
          level: normalizeLessonLevel(readString(node.level) ?? readString(artifact.metadata?.difficulty)),
        },
      ],
    })),
    studyResources: [],
  };
}

async function readLatestPersistedRoadmap(config: GatewayConfig, userId?: string): Promise<Record<string, unknown> | null> {
  const baseUrl = config.databaseMcpBaseUrl.replace(/\/$/, "");
  const query = userId ? `?userId=${encodeURIComponent(userId)}` : "";
  const response = await fetch(`${baseUrl}/roadmaps/latest${query}`);
  if (!response.ok) {
    throw new Error(`Database MCP returned ${response.status}`);
  }

  const body = await response.json();
  return isRecord(body?.roadmap) ? body.roadmap : null;
}

async function readPersistedNotes(config: GatewayConfig, userId?: string): Promise<Record<string, unknown>[]> {
  const baseUrl = config.databaseMcpBaseUrl.replace(/\/$/, "");
  const query = userId ? `?userId=${encodeURIComponent(userId)}` : "";
  const response = await fetch(`${baseUrl}/notes${query}`);
  if (!response.ok) {
    throw new Error(`Database MCP returned ${response.status}`);
  }

  const body = await response.json();
  return Array.isArray(body?.notes) ? body.notes.filter(isRecord) : [];
}

async function createPersistedNote(
  config: GatewayConfig,
  input: { userId: string; content: string; taskId?: string },
): Promise<Record<string, unknown>> {
  const baseUrl = config.databaseMcpBaseUrl.replace(/\/$/, "");
  const response = await fetch(`${baseUrl}/notes`, {
    method: "POST",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify(input),
  });
  if (!response.ok) {
    throw new Error(`Database MCP returned ${response.status}`);
  }

  const body = await response.json();
  return isRecord(body?.note) ? body.note : {};
}

async function readPersistedReviewItems(config: GatewayConfig, userId?: string): Promise<Record<string, unknown>[]> {
  const baseUrl = config.databaseMcpBaseUrl.replace(/\/$/, "");
  const query = userId ? `?userId=${encodeURIComponent(userId)}` : "";
  const response = await fetch(`${baseUrl}/review${query}`);
  if (!response.ok) {
    throw new Error(`Database MCP returned ${response.status}`);
  }

  const body = await response.json();
  return Array.isArray(body?.reviewItems) ? dedupeReviewItems(body.reviewItems.filter(isRecord)) : [];
}

function persistedNoteToPageData(note: Record<string, unknown>) {
  const content = readString(note.content) ?? "";
  const title = readString(note.task_title) ?? firstContentLine(content) ?? "Untitled note";
  const course = readString(note.project_title) ?? "General";
  const updatedAt = readString(note.created_at) ?? new Date().toISOString().slice(0, 10);

  return {
    id: readString(note.id) ?? `note-${hashText(content)}`,
    noteId: readString(note.id),
    taskId: readString(note.task_id),
    title,
    excerpt: summarizeText(content, 140),
    content,
    course,
    tags: [course, readString(note.task_title)].filter(Boolean),
    updatedAt: updatedAt.slice(0, 10),
    pinned: false,
    tone: "info",
  };
}

function persistedReviewItemToPageData(item: Record<string, unknown>) {
  const progressPercent = readNumber(item.progress_percent) ?? 0;
  const status = readString(item.status) ?? "pending";
  return {
    id: readString(item.id) ?? readString(item.task_id) ?? `review-${hashText(readString(item.title) ?? "")}`,
    taskId: readString(item.task_id) ?? readString(item.id),
    concept: readString(item.title) ?? "Untitled review task",
    course: readString(item.project_title) ?? "General",
    due: status === "needs_review" ? "Hôm nay" : "Sắp tới",
    confidence: Math.max(0, Math.min(1, progressPercent / 100)),
    tone: status === "needs_review" ? "warning" : progressPercent < 50 ? "risk" : "info",
  };
}

function dedupeReviewItems(items: Record<string, unknown>[]) {
  const seen = new Set<string>();
  return items.filter((item) => {
    const key = [
      readString(item.project_title)?.toLowerCase() ?? "general",
      readString(item.title)?.toLowerCase() ?? readString(item.task_id) ?? readString(item.id) ?? "",
    ].join("::");
    if (seen.has(key)) {
      return false;
    }
    seen.add(key);
    return true;
  });
}

function persistedRoadmapToPageData(roadmap: Record<string, unknown>) {
  const phases = Array.isArray(roadmap.phases) ? roadmap.phases.filter(isRecord) : [];
  const pagePhases = phases.map((phase, index) => persistedPhaseToCard(phase, index));
  const totalTasks = pagePhases.reduce((sum, phase) => sum + phase.taskCount, 0);
  const completedTasks = pagePhases.reduce((sum, phase) => sum + phase.completedTaskCount, 0);

  return {
    roadmapSummary: {
      id: readString(roadmap.id),
      roadmapId: readString(roadmap.id),
      title: readString(roadmap.title) ?? readString(roadmap.project_title) ?? "Learning roadmap",
      description: readString(roadmap.project_description) ?? "Roadmap saved in the learning database.",
      progress: totalTasks > 0 ? completedTasks / totalTasks : 0,
      currentSprint: readString(roadmap.generated_by) ?? "database",
      nextMilestone: pagePhases.find((phase) => phase.status !== "completed")?.title ?? pagePhases[0]?.title ?? "No milestone selected",
    },
    roadmapPhases: pagePhases.map(({ taskCount, completedTaskCount, ...phase }) => phase),
    studyResources: [],
  };
}

function persistedPhaseToCard(phase: Record<string, unknown>, index: number) {
  const milestones = Array.isArray(phase.milestones) ? phase.milestones.filter(isRecord) : [];
  const tasks = milestones.flatMap((milestone) => (Array.isArray(milestone.tasks) ? milestone.tasks.filter(isRecord) : []));
  const completedTaskCount = tasks.filter((task) => readString(task.status) === "completed").length;
  const taskCount = tasks.length;
  const progress = taskCount > 0 ? completedTaskCount / taskCount : 0;
  const status = progress >= 1 ? "completed" : progress > 0 ? "in-progress" : "not-started";

  return {
    id: readString(phase.id) ?? `phase-${index + 1}`,
    roadmapId: readString(phase.roadmap_id),
    phaseId: readString(phase.id),
    title: readString(phase.title) ?? `Phase ${index + 1}`,
    duration: readNumber(phase.estimated_days) ? `${readNumber(phase.estimated_days)} days` : `Phase ${index + 1}`,
    status,
    statusLabel: status,
    tone: status === "completed" ? "success" : status === "in-progress" ? "info" : "neutral",
    progress,
    tasks: tasks.length
      ? tasks.map((task) => ({
          id: readString(task.id),
          taskId: readString(task.id),
          roadmapId: readString(phase.roadmap_id),
          milestoneId: readString(task.milestone_id),
          title: readString(task.title) ?? "Untitled task",
          description: readString(task.description),
          status: readString(task.status),
          difficulty: readString(task.difficulty),
          level: normalizeLessonLevel(readString(task.difficulty)),
        }))
      : milestones.map((milestone) => ({
          id: readString(milestone.id),
          milestoneId: readString(milestone.id),
          roadmapId: readString(phase.roadmap_id),
          title: readString(milestone.title) ?? "Untitled milestone",
          description: readString(milestone.description),
          level: normalizeLessonLevel(readString(milestone.difficulty)),
        })),
    taskCount,
    completedTaskCount,
  };
}

function uniqueLabels(values: string[]) {
  return [...new Set(values.filter(Boolean))];
}

function readString(value: unknown) {
  return typeof value === "string" && value.trim() ? value.trim() : undefined;
}

function readNumber(value: unknown) {
  return typeof value === "number" && Number.isFinite(value) ? value : undefined;
}

function firstContentLine(value: string) {
  return value
    .split(/\r?\n/)
    .map((line) => line.trim())
    .find(Boolean);
}

function summarizeText(value: string, maxLength: number) {
  const normalized = value.replace(/\s+/g, " ").trim();
  if (normalized.length <= maxLength) {
    return normalized;
  }
  return `${normalized.slice(0, maxLength - 3).trim()}...`;
}

function hashText(value: string) {
  let hash = 0;
  for (const char of value) {
    hash = (hash * 31 + char.charCodeAt(0)) >>> 0;
  }
  return hash.toString(16);
}

function isRecord(value: unknown): value is Record<string, unknown> {
  return Boolean(value) && typeof value === "object" && !Array.isArray(value);
}

function normalizeResourceType(value?: string) {
  if (!value) {
    return "Article";
  }
  const normalized = value.replace(/[_-]+/g, " ");
  return normalized.charAt(0).toUpperCase() + normalized.slice(1);
}

function normalizeLessonLevel(value?: string) {
  const normalized = value?.trim().toLowerCase();
  if (normalized === "beginner" || normalized === "easy") {
    return "beginner";
  }
  if (normalized === "intermediate" || normalized === "medium") {
    return "intermediate";
  }
  if (normalized === "advanced" || normalized === "hard" || normalized === "expert") {
    return "advanced";
  }
  return undefined;
}
