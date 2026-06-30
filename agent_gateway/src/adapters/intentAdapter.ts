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
    case "roadmap.schedule_update.requested":
      return [
        `Update the study schedule for roadmap ${intent.payload.roadmapId}.`,
        intent.payload.title ? `Roadmap title: ${intent.payload.title}` : undefined,
        intent.payload.nextMilestone ? `Next milestone: ${intent.payload.nextMilestone}` : undefined,
        "Return concise next steps and any schedule adjustment needed.",
      ]
        .filter(Boolean)
        .join("\n");
    case "roadmap.task.selected":
      return [
        `Load or generate a lesson for roadmap ${intent.payload.roadmapId}, task ${intent.payload.taskId}.`,
        intent.payload.phaseId ? `Phase: ${intent.payload.phaseId}` : undefined,
        intent.payload.milestoneId ? `Milestone: ${intent.payload.milestoneId}` : undefined,
        intent.payload.level ? `Lesson level: ${intent.payload.level}` : undefined,
        `Task title: ${intent.payload.title}`,
        intent.payload.description ? `Task description: ${intent.payload.description}` : undefined,
        "Return a UI-safe lesson artifact if possible.",
      ]
        .filter(Boolean)
        .join("\n");
    case "note.review.requested":
      return [
        "Create a focused review session from this saved note.",
        intent.payload.noteId ? `Note: ${intent.payload.noteId}` : undefined,
        intent.payload.taskId ? `Related task: ${intent.payload.taskId}` : undefined,
        `Title: ${intent.payload.title}`,
        intent.payload.course ? `Course: ${intent.payload.course}` : undefined,
        intent.payload.tags?.length ? `Tags: ${intent.payload.tags.join(", ")}` : undefined,
        intent.payload.content ? `Note content: ${intent.payload.content}` : undefined,
      ]
        .filter(Boolean)
        .join("\n");
    case "review.task.selected":
      return [
        "Start a spaced-review session for this task.",
        intent.payload.taskId ? `Task: ${intent.payload.taskId}` : undefined,
        `Concept: ${intent.payload.concept}`,
        intent.payload.course ? `Course: ${intent.payload.course}` : undefined,
        typeof intent.payload.confidence === "number" ? `Current confidence: ${intent.payload.confidence}` : undefined,
        intent.payload.due ? `Due: ${intent.payload.due}` : undefined,
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
    case "roadmap.schedule_update.requested":
      return `Update roadmap schedule ${intent.payload.roadmapId}`;
    case "roadmap.task.selected":
      return `Study task ${intent.payload.title}`;
    case "note.review.requested":
      return `Create review from note ${intent.payload.title}`;
    case "review.task.selected":
      return `Review ${intent.payload.concept}`;
  }
}

export function intentToOrchestratorContext(intent: UserIntent) {
  switch (intent.type) {
    case "goal.submitted":
      return {
        type: intent.type,
        goal: intent.payload.goal,
        level: intent.payload.level,
        durationWeeks: intent.payload.durationWeeks,
        hoursPerWeek: intent.payload.hoursPerWeek,
        preferredStyle: intent.payload.preferredStyle,
        constraints: intent.payload.constraints,
      };
    case "chat.submitted":
      return {
        type: intent.type,
        message: intent.payload.message,
        contextArtifactId: intent.payload.contextArtifactId,
      };
    case "roadmap.node.selected":
      return {
        type: intent.type,
        roadmapId: intent.payload.roadmapId,
        nodeId: intent.payload.nodeId,
        roadmapNodeId: intent.payload.nodeId,
      };
    case "lesson.answer.submitted":
      return {
        type: intent.type,
        lessonId: intent.payload.lessonId,
        exerciseId: intent.payload.exerciseId,
        answer: intent.payload.answer,
      };
    case "resource.backfill.requested":
      return {
        type: intent.type,
        topicId: intent.payload.topicId,
        reason: intent.payload.reason,
        priority: intent.payload.priority,
      };
    case "roadmap.regenerate.requested":
      return {
        type: intent.type,
        roadmapId: intent.payload.roadmapId,
        reason: intent.payload.reason,
        preserveCompletedNodes: intent.payload.preserveCompletedNodes,
      };
    case "roadmap.schedule_update.requested":
      return {
        type: intent.type,
        roadmapId: intent.payload.roadmapId,
        title: intent.payload.title,
        nextMilestone: intent.payload.nextMilestone,
      };
    case "roadmap.task.selected":
      return {
        type: intent.type,
        roadmapId: intent.payload.roadmapId,
        phaseId: intent.payload.phaseId,
        milestoneId: intent.payload.milestoneId,
        taskId: intent.payload.taskId,
        nodeId: intent.payload.taskId,
        roadmapNodeId: intent.payload.taskId,
        title: intent.payload.title,
        description: intent.payload.description,
        level: intent.payload.level,
      };
    case "note.review.requested":
      return {
        type: intent.type,
        noteId: intent.payload.noteId,
        taskId: intent.payload.taskId,
        nodeId: intent.payload.taskId,
        roadmapNodeId: intent.payload.taskId,
        title: intent.payload.title,
        course: intent.payload.course,
        tags: intent.payload.tags,
      };
    case "review.task.selected":
      return {
        type: intent.type,
        taskId: intent.payload.taskId,
        nodeId: intent.payload.taskId,
        roadmapNodeId: intent.payload.taskId,
        concept: intent.payload.concept,
        course: intent.payload.course,
        confidence: intent.payload.confidence,
        due: intent.payload.due,
      };
  }
}
