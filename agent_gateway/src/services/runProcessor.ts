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
import type { TrustedAuthContext } from "./authContext.js";

export class RunProcessor {
  private readonly orchestratorClient: OrchestratorClient;
  private readonly controllers = new Map<string, AbortController>();

  constructor(config: GatewayConfig) {
    this.orchestratorClient = new OrchestratorClient(config);
  }

  start(input: { sessionId: string; runId: string; intent: UserIntent; authContext?: TrustedAuthContext }) {
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

  startResume(input: {
    sessionId: string;
    runId: string;
    stepId: string;
    decision: "approve" | "reject" | "revise";
    comment?: string;
    authContext?: TrustedAuthContext;
  }) {
    setImmediate(() => {
      this.executeResume(input).catch((error) => {
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

  private async execute(input: { sessionId: string; runId: string; intent: UserIntent; authContext?: TrustedAuthContext }) {
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
      userId: input.authContext?.userId ?? sessionStore.getSession(input.sessionId)?.userId,
      authContext: input.authContext,
      signal: controller.signal,
    });

    eventBus.publish(input.sessionId, input.runId, {
      type: "tool.completed",
      toolName: "orchestrator.run",
      displayName: "Running Orchestrator Agent",
      resultSummary: response.message,
    });

    maybeRequireApprovalAction(input.sessionId, input.runId, response);
    if (response.status === "waiting_for_user") {
      sessionStore.updateRun(input.sessionId, input.runId, {
        status: "waiting_for_user",
        currentStep: undefined,
      });
      eventBus.publish(input.sessionId, input.runId, {
        type: "run.status_changed",
        status: "waiting_for_user",
      });
      this.controllers.delete(input.runId);
      return;
    }

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

  private async executeResume(input: {
    sessionId: string;
    runId: string;
    stepId: string;
    decision: "approve" | "reject" | "revise";
    comment?: string;
    authContext?: TrustedAuthContext;
  }) {
    const controller = new AbortController();
    this.controllers.set(input.runId, controller);

    sessionStore.updateRun(input.sessionId, input.runId, {
      status: "running",
      currentStep: "Resuming Orchestrator approval gate",
    });
    eventBus.publish(input.sessionId, input.runId, {
      type: "run.status_changed",
      status: "running",
      currentStep: "Resuming Orchestrator approval gate",
    });
    eventBus.publish(input.sessionId, input.runId, {
      type: "tool.started",
      toolName: "orchestrator.resume",
      displayName: "Resuming Orchestrator Agent",
    });

    const response = await this.orchestratorClient.resume({
      sessionId: input.sessionId,
      userId: input.authContext?.userId ?? sessionStore.getSession(input.sessionId)?.userId,
      authContext: input.authContext,
      approval: {
        stepId: input.stepId,
        decision: input.decision,
        comment: input.comment,
      },
      signal: controller.signal,
    });

    eventBus.publish(input.sessionId, input.runId, {
      type: "tool.completed",
      toolName: "orchestrator.resume",
      displayName: "Resuming Orchestrator Agent",
      resultSummary: response.message,
    });
    maybeRequireApprovalAction(input.sessionId, input.runId, response);
    if (response.status === "waiting_for_user") {
      sessionStore.updateRun(input.sessionId, input.runId, {
        status: "waiting_for_user",
        currentStep: undefined,
      });
      eventBus.publish(input.sessionId, input.runId, {
        type: "run.status_changed",
        status: "waiting_for_user",
      });
      this.controllers.delete(input.runId);
      return;
    }

    const normalized = normalizeOrchestratorOutput(response.output, `Resume approval step ${input.stepId}`);
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
        message: normalized.summary ?? "Agent resume output was converted into UI artifacts.",
      });
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
    const finalStatus = hasPendingActions ? "waiting_for_user" : response.status === "rejected" ? "failed" : "completed";
    sessionStore.updateRun(input.sessionId, input.runId, {
      status: finalStatus,
      currentStep: undefined,
      completedAt: finalStatus === "completed" || finalStatus === "failed" ? nowIso() : undefined,
    });
    eventBus.publish(input.sessionId, input.runId, {
      type: "run.status_changed",
      status: finalStatus,
    });
    this.controllers.delete(input.runId);
  }
}

function maybeRequireApprovalAction(sessionId: string, runId: string, response: { status?: string; output?: unknown }) {
  if (response.status !== "waiting_for_user") {
    return;
  }

  const output = isRecord(response.output) ? response.output : {};
  const approval = isRecord(output.approval) ? output.approval : {};
  const stepId = typeof approval.step_id === "string" ? approval.step_id : typeof approval.stepId === "string" ? approval.stepId : undefined;
  if (!stepId) {
    return;
  }

  eventBus.publish(sessionId, runId, {
    type: "ui.action_required",
    action: {
      id: makeId("action"),
      runId,
      title: "Orchestrator needs approval",
      description:
        typeof approval.question === "string" ? approval.question : "Review the pending agent step before continuing.",
      options: [
        { id: "approve", label: "Approve", variant: "primary" },
        { id: "revise", label: "Revise", variant: "secondary" },
        { id: "reject", label: "Reject", variant: "danger" },
      ],
      status: "pending",
      createdAt: nowIso(),
      metadata: {
        kind: "orchestrator_approval",
        stepId,
      },
    },
  });
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

function isRecord(value: unknown): value is Record<string, unknown> {
  return typeof value === "object" && value !== null && !Array.isArray(value);
}
