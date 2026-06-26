export const roadmapSummary = {
  title: "AI Engineer trong 12 tuần",
  description:
    "Lộ trình ưu tiên nền tảng ML, RAG, agent workflow và MLOps nhẹ để phục vụ đồ án self-learning.",
  progress: 0.46,
  currentSprint: "Tuần 5",
  nextMilestone: "Hoàn thiện RAG evaluation",
};

export const roadmapPhases = [
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
  {
    id: "agent",
    title: "Agent workflow",
    duration: "Tuần 7-9",
    status: "queued",
    statusLabel: "Sắp tới",
    tone: "agent",
    progress: 0.08,
    tasks: ["Tool calling", "Planner loop", "Memory strategy"],
  },
  {
    id: "delivery",
    title: "MLOps & demo",
    duration: "Tuần 10-12",
    status: "queued",
    statusLabel: "Sắp tới",
    tone: "teal",
    progress: 0,
    tasks: ["Experiment tracking", "Evaluation report", "Demo deployment"],
  },
];

export const studyResources = [
  {
    id: "resource-01",
    title: "RAG evaluation checklist",
    type: "Checklist",
    tone: "info",
    source: "Mentor agent",
  },
  {
    id: "resource-02",
    title: "Embedding search notes",
    type: "Note",
    tone: "teal",
    source: "Saved from chat",
  },
  {
    id: "resource-03",
    title: "Agent tool calling patterns",
    type: "Guide",
    tone: "agent",
    source: "Course material",
  },
];
