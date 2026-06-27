export const resources = [
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
  {
    id: "resource-mlops-template",
    title: "Experiment report template",
    description: "Mẫu báo cáo experiment gồm config, metric, artifact và conclusion.",
    type: "Template",
    source: "Resource library",
    course: "MLOps Starter",
    saved: false,
    tone: "success",
    updatedAt: "2026-06-22",
  },
];

export const resourceCourses = ["Tất cả", "RAG Foundation", "AI Agent Workflow", "MLOps Starter"];

export const resourceTypes = ["Tất cả", "Checklist", "Guide", "Note", "Template"];
