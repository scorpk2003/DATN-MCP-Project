export const learner = {
  name: "Khang",
  focus: "AI Engineer roadmap",
  streakDays: 12,
  weeklyGoalHours: 18,
  completedHours: 11,
};

export const recentChats = [
  { id: "chat-01", label: "Lập kế hoạch học RAG" },
  { id: "chat-02", label: "Ôn lại vector database" },
  { id: "chat-03", label: "Giải thích backpropagation" },
  { id: "chat-04", label: "Checklist đồ án tốt nghiệp" },
];

export const quickPrompts = [
  "Tạo roadmap 30 ngày",
  "Tóm tắt bài vừa học",
  "Sinh câu hỏi ôn tập",
  "Gợi ý tài liệu tiếp theo",
];

export const learningMetrics = [
  { id: "progress", label: "Tiến độ tuần", value: "61%", tone: "success" },
  { id: "focus", label: "Phiên tập trung", value: "8", tone: "info" },
  { id: "review", label: "Cần ôn tập", value: "14", tone: "warning" },
];

export const recommendedCourses = [
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

export const agentActivities = [
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
