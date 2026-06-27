import type { UserIntent } from "../protocol/index.js";

export function intentToGoal(intent: UserIntent) {
  switch (intent.type) {
    case "goal.submitted": {
      const parts = [
        `Create a learning roadmap for this goal: ${intent.payload.goal}`,
        intent.payload.level ? `Learner level: ${intent.payload.level}` : undefined,
        intent.payload.durationWeeks ? `Duration: ${intent.payload.durationWeeks} weeks` : undefined,
        intent.payload.hoursPerWeek ? `Available time: ${intent.payload.hoursPerWeek} hours per week` : undefined,
        intent.payload.preferredStyle?.length
          ? `Preferred learning styles: ${intent.payload.preferredStyle.join(", ")}`
          : undefined,
        intent.payload.constraints?.length ? `Constraints: ${intent.payload.constraints.join("; ")}` : undefined,
      ];
      return parts.filter(Boolean).join("\n");
    }
    case "chat.submitted":
      return [
        "Answer this learner message in the current self-learning session.",
        intent.payload.contextArtifactId ? `Context artifact: ${intent.payload.contextArtifactId}` : undefined,
        `Message: ${intent.payload.message}`,
      ]
        .filter(Boolean)
        .join("\n");
    case "roadmap.node.selected":
      return `Load or generate a lesson for roadmap ${intent.payload.roadmapId}, node ${intent.payload.nodeId}. Return a UI-safe lesson artifact if possible.`;
    case "lesson.answer.submitted":
      return [
        `Grade the learner answer for lesson ${intent.payload.lessonId}.`,
        intent.payload.exerciseId ? `Exercise: ${intent.payload.exerciseId}` : undefined,
        `Answer: ${intent.payload.answer}`,
      ]
        .filter(Boolean)
        .join("\n");
    case "resource.backfill.requested":
      return [
        `Backfill learning resources for topic ${intent.payload.topicId}.`,
        intent.payload.priority ? `Priority: ${intent.payload.priority}` : undefined,
        intent.payload.reason ? `Reason: ${intent.payload.reason}` : undefined,
      ]
        .filter(Boolean)
        .join("\n");
    case "roadmap.regenerate.requested":
      return [
        `Regenerate roadmap ${intent.payload.roadmapId}.`,
        intent.payload.preserveCompletedNodes === true ? "Preserve completed nodes." : undefined,
        intent.payload.reason ? `Reason: ${intent.payload.reason}` : undefined,
      ]
        .filter(Boolean)
        .join("\n");
  }
}

export function intentToUserMessage(intent: UserIntent) {
  switch (intent.type) {
    case "goal.submitted":
      return intent.payload.goal;
    case "chat.submitted":
      return intent.payload.message;
    case "roadmap.node.selected":
      return `Open roadmap node ${intent.payload.nodeId}`;
    case "lesson.answer.submitted":
      return intent.payload.answer;
    case "resource.backfill.requested":
      return `Backfill resources for ${intent.payload.topicId}`;
    case "roadmap.regenerate.requested":
      return `Regenerate roadmap ${intent.payload.roadmapId}`;
  }
}
