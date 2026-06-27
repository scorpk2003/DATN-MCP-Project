export const notes = [
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
  {
    id: "note-tool-schema",
    title: "Tool schema design",
    excerpt: "Schema tốt giúp agent chọn tool chính xác và giảm lỗi tham số.",
    content:
      "Tool schema nên đặt tên rõ, mô tả ngắn, field bắt buộc tối thiểu và enum khi miền giá trị hữu hạn. Với tool phức tạp, tách thành nhiều action nhỏ dễ quan sát hơn.",
    course: "AI Agent Workflow",
    tags: ["agent", "tooling"],
    updatedAt: "2026-06-24",
    pinned: true,
    tone: "agent",
  },
  {
    id: "note-experiment",
    title: "Experiment tracking",
    excerpt: "Ghi lại config, metric và artifact để so sánh các lần chạy.",
    content:
      "Một experiment nên có dataset version, model/config, metric chính, log lỗi và artifact đầu ra. Việc này giúp demo đồ án đáng tin hơn.",
    course: "MLOps Starter",
    tags: ["mlops", "tracking"],
    updatedAt: "2026-06-22",
    pinned: false,
    tone: "teal",
  },
];

export const noteCourses = ["Tất cả", "RAG Foundation", "AI Agent Workflow", "MLOps Starter"];
