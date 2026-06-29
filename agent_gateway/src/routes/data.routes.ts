import { Router } from "express";
import type { GatewayConfig } from "../config.js";
import { asyncHandler } from "../middleware/asyncHandler.js";

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
    "/navigation",
    asyncHandler(async (_request, response) => {
      response.json({ recentChats, sidebarItems });
    }),
  );

  router.get(
    "/resources",
    asyncHandler(async (_request, response) => {
      const resources: GatewayResource[] = await readResourcesFromResourceService(config).catch(() => resourcesFallback);
      response.json({
        resources,
        resourceCourses: uniqueLabels(["Tất cả", ...resources.map((resource) => resource.course)]),
        resourceTypes: uniqueLabels(["Tất cả", ...resources.map((resource) => resource.type)]),
      });
    }),
  );

  router.get(
    "/notes",
    asyncHandler(async (_request, response) => {
      response.json({
        notes,
        noteCourses: uniqueLabels(["Tất cả", ...notes.map((note) => note.course)]),
      });
    }),
  );

  router.get(
    "/review",
    asyncHandler(async (_request, response) => {
      response.json({ reviewQueue });
    }),
  );

  router.get(
    "/roadmap",
    asyncHandler(async (_request, response) => {
      response.json({
        roadmapSummary,
        roadmapPhases,
        studyResources: resourcesFallback.slice(0, 3).map((resource) => ({
          id: resource.id,
          title: resource.title,
          type: resource.type,
          tone: resource.tone,
          source: resource.source,
        })),
      });
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
  const items = Array.isArray(body) ? body : Array.isArray(body.resources) ? body.resources : Array.isArray(body.items) ? body.items : [];
  if (!items.length) {
    return resourcesFallback;
  }

  return items.slice(0, 100).map((item: Record<string, unknown>, index: number) => ({
    id: readString(item.id) ?? `resource-${index + 1}`,
    title: readString(item.title) ?? "Untitled resource",
    description: readString(item.description) ?? readString(item.summary) ?? "No description available.",
    type: normalizeResourceType(readString(item.type) ?? readString(item.kind)),
    source: readString(item.source) ?? readString(item.publisher) ?? readString(item.primary_domain) ?? "Resource Service",
    course: readString(item.course) ?? readString(item.topic) ?? "General",
    saved: Boolean(item.saved),
    tone: "info",
    updatedAt: readString(item.updatedAt) ?? readString(item.updated_at) ?? readString(item.created_at) ?? new Date().toISOString().slice(0, 10),
    url: readString(item.url) ?? readString(item.canonical_url),
  }));
}

function uniqueLabels(values: string[]) {
  return [...new Set(values.filter(Boolean))];
}

function readString(value: unknown) {
  return typeof value === "string" && value.trim() ? value.trim() : undefined;
}

function normalizeResourceType(value?: string) {
  if (!value) {
    return "Article";
  }
  const normalized = value.replace(/[_-]+/g, " ");
  return normalized.charAt(0).toUpperCase() + normalized.slice(1);
}
