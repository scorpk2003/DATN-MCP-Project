import type { GatewayConfig } from "../config.js";
import { intentToGoal } from "../adapters/intentAdapter.js";
import { normalizeOrchestratorOutput } from "../adapters/orchestratorOutputAdapter.js";
import type { UserIntent } from "../protocol/index.js";
import { GatewayError } from "./errors.js";
import { eventBus } from "./eventBus.js";
import { makeId, nowIso } from "./id.js";
import { OrchestratorClient } from "./orchestratorClient.js";
import { sessionStore } from "./sessionStore.js";
import { validateArtifact } from "./artifactValidator.js";

export class RunProcessor {
  private readonly orchestratorClient: OrchestratorClient;
  private readonly controllers = new Map<string, AbortController>();

  constructor(config: GatewayConfig) {
    this.orchestratorClient = new OrchestratorClient(config);
  }

  start(input: { sessionId: string; runId: string; intent: UserIntent }) {
    setImmediate(() => {
      this.execute(input).catch((error) => {
        const normalized = normalizeRunError(error);
        sessionStore.updateRun(input.sessionId, input.runId, {
          status: "failed",
          completedAt: nowIso(),
          error: normalized.message,
        });
        eventBus.publish(input.sessionId, input.runId, {
          type: "error",
          message: normalized.message,
          recoverable: true,
          code: normalized.code,
        });
        eventBus.publish(input.sessionId, input.runId, {
          type: "run.status_changed",
          status: "failed",
        });
      });
    });
  }

  cancel(sessionId: string, runId: string) {
    const run = sessionStore.getRun(sessionId, runId);
    if (!run) {
      return null;
    }

    if (["completed", "failed", "cancelled"].includes(run.status)) {
      return run;
    }

    this.controllers.get(runId)?.abort();
    this.controllers.delete(runId);
    const cancelled = sessionStore.updateRun(sessionId, runId, {
      status: "cancelled",
      completedAt: nowIso(),
    });
    eventBus.publish(sessionId, runId, {
      type: "run.status_changed",
      status: "cancelled",
    });
    return cancelled;
  }

  private async execute(input: { sessionId: string; runId: string; intent: UserIntent }) {
    const goal = intentToGoal(input.intent);
    const controller = new AbortController();
    this.controllers.set(input.runId, controller);

    sessionStore.updateRun(input.sessionId, input.runId, {
      status: "running",
      currentStep: "Preparing orchestrator request",
    });

    eventBus.publish(input.sessionId, input.runId, {
      type: "run.status_changed",
      status: "running",
      currentStep: "Preparing orchestrator request",
    });
    eventBus.publish(input.sessionId, input.runId, {
      type: "agent.message",
      message: "Mình đã nhận mục tiêu và sẽ chuyển cho Orchestrator xử lý.",
    });
    eventBus.publish(input.sessionId, input.runId, {
      type: "agent.thinking",
      label: "Preparing typed intent for Orchestrator",
    });
    eventBus.publish(input.sessionId, input.runId, {
      type: "tool.started",
      toolName: "orchestrator.run",
      displayName: "Running Orchestrator Agent",
    });

    const response = await this.orchestratorClient.run({
      goal,
      sessionId: input.sessionId,
      signal: controller.signal,
    });

    eventBus.publish(input.sessionId, input.runId, {
      type: "tool.completed",
      toolName: "orchestrator.run",
      displayName: "Running Orchestrator Agent",
      resultSummary: response.message,
    });

    const normalized = normalizeOrchestratorOutput(response.output, goal);
    if (normalized.type === "artifacts") {
      for (const artifact of normalized.artifacts) {
        const validated = validateArtifact(artifact);
        eventBus.publish(input.sessionId, input.runId, {
          type: "artifact.created",
          artifact: validated,
        });
      }
      eventBus.publish(input.sessionId, input.runId, {
        type: "agent.message",
        message: normalized.summary ?? "Agent output was converted into UI artifacts.",
      });
      maybeRequireRoadmapAction(input.sessionId, input.runId, normalized.artifacts[0]?.kind === "roadmap" ? normalized.artifacts[0] : null);
    } else {
      eventBus.publish(input.sessionId, input.runId, {
        type: "agent.message",
        message: normalized.message,
      });
      sessionStore.updateRun(input.sessionId, input.runId, {
        metadata: {
          rawOutput: response.output,
        },
      });
    }

    const state = sessionStore.getState(input.sessionId);
    const hasPendingActions = state?.pendingActions.some((action) => action.runId === input.runId);
    const finalStatus = hasPendingActions ? "waiting_for_user" : "completed";
    sessionStore.updateRun(input.sessionId, input.runId, {
      status: finalStatus,
      currentStep: undefined,
      completedAt: finalStatus === "completed" ? nowIso() : undefined,
    });
    eventBus.publish(input.sessionId, input.runId, {
      type: "run.status_changed",
      status: finalStatus,
    });
    this.controllers.delete(input.runId);
  }
}

function maybeRequireRoadmapAction(sessionId: string, runId: string, artifact: { coverageStatus: string } | null) {
  if (!artifact || artifact.coverageStatus === "good") {
    return;
  }

  eventBus.publish(sessionId, runId, {
    type: "ui.action_required",
    action: {
      id: makeId("action"),
      runId,
      title: "Resource coverage needs review",
      description: "Some roadmap nodes may need stronger learning resources before starting.",
      options: [
        { id: "start_anyway", label: "Start anyway", variant: "secondary" },
        { id: "backfill_first", label: "Backfill resources first", variant: "primary" },
        { id: "edit_goal", label: "Edit goal", variant: "secondary" },
      ],
      status: "pending",
      createdAt: nowIso(),
    },
  });
}

function normalizeRunError(error: unknown) {
  if (error instanceof GatewayError) {
    return {
      code: error.code,
      message: error.message,
    };
  }

  return {
    code: "ORCHESTRATOR_FAILED",
    message: "Agent run failed before producing a UI-safe result.",
  };
}
