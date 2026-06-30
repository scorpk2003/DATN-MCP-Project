import assert from "node:assert/strict";
import { describe, it } from "node:test";
import { normalizeOrchestratorOutput } from "../adapters/orchestratorOutputAdapter.js";

describe("orchestrator output adapter", () => {
  it("does not emit roadmap artifacts without a real roadmap id", () => {
    const normalized = normalizeOrchestratorOutput(
      {
        nodes: [{ id: "node-a", title: "Intro" }],
      },
      "Learn Rust",
    );

    assert.equal(normalized.type, "message");
  });

  it("uses intent context for lesson workflow identifiers", () => {
    const normalized = normalizeOrchestratorOutput(
      {
        lessonDraft: {
          id: "lesson-a",
          title: "Intro",
          explanation: "Start here.",
          resources: [],
        },
      },
      "Open lesson",
      {
        roadmapId: "roadmap-a",
        roadmapNodeId: "node-a",
      },
    );

    assert.equal(normalized.type, "artifacts");
    assert.equal(normalized.artifacts[0]?.kind, "lesson");
    assert.equal(normalized.artifacts[0]?.id, "lesson-a");
    if (normalized.artifacts[0]?.kind !== "lesson") {
      throw new Error("expected lesson artifact");
    }
    assert.equal(normalized.artifacts[0].roadmapId, "roadmap-a");
    assert.equal(normalized.artifacts[0].nodeId, "node-a");
  });

  it("creates lesson artifacts for review lesson drafts without roadmap ids", () => {
    const normalized = normalizeOrchestratorOutput(
      {
        lessonDraft: {
          title: "Core concept 1 review",
          topic: "Core concept 1",
          objectives: ["Review the weak concept."],
          contentBlocks: [
            {
              id: "block-1",
              type: "explanation",
              title: "Key idea",
              content: "Use this review pass to close the concept gap.",
            },
          ],
          resources: [
            {
              id: "resource-1",
              title: "Official review notes",
              sourceType: "docs",
              trustTier: 1,
            },
          ],
          exercises: [
            {
              id: "exercise-1",
              prompt: "Explain the concept in your own words.",
              difficulty: "easy",
            },
          ],
        },
      },
      "Start a spaced-review session for this task.",
      {
        type: "review.task.selected",
        taskId: "task-core-1",
        nodeId: "task-core-1",
        roadmapNodeId: "task-core-1",
        concept: "Core concept 1",
      },
    );

    assert.equal(normalized.type, "artifacts");
    assert.equal(normalized.artifacts[0]?.kind, "lesson");
    if (normalized.artifacts[0]?.kind !== "lesson") {
      throw new Error("expected lesson artifact");
    }
    assert.equal(normalized.artifacts[0].roadmapId, "review_task-core-1");
    assert.equal(normalized.artifacts[0].nodeId, "task-core-1");
    assert.equal(normalized.artifacts[0].explanation, "Use this review pass to close the concept gap.");
    assert.equal(normalized.artifacts[0].resources[0]?.sourceType, "official_docs");
  });

  it("creates a review shell when approval completes without generated content", () => {
    const normalized = normalizeOrchestratorOutput(
      {
        ok: true,
        status: "completed",
        session_id: "session-a",
        output: {},
      },
      "Start a spaced-review session for this task.",
      {
        type: "review.task.selected",
        taskId: "task-core-1",
        nodeId: "task-core-1",
        roadmapNodeId: "task-core-1",
        concept: "Core concept 1",
        course: "Generated roadmap",
        confidence: 0.35,
      },
    );

    assert.equal(normalized.type, "artifacts");
    assert.equal(normalized.artifacts[0]?.kind, "lesson");
    if (normalized.artifacts[0]?.kind !== "lesson") {
      throw new Error("expected lesson artifact");
    }
    assert.equal(normalized.artifacts[0].roadmapId, "review_task-core-1");
    assert.equal(normalized.artifacts[0].nodeId, "task-core-1");
    assert.equal(normalized.artifacts[0].title, "Core concept 1 review");
  });
});
