import type { GatewayConfig } from "../config.js";
import { GatewayError } from "./errors.js";

export type OrchestratorResponse = {
  success?: boolean;
  ok?: boolean;
  status?: string;
  session_id?: string;
  output?: unknown;
  message?: string;
  error?: {
    code?: string;
    message?: string;
    recoverable?: boolean;
  };
};

export type OrchestratorAuthContext = {
  userId: string;
  verified: boolean;
  scope: string[];
  verifiedBy: string;
  verifiedAt: string;
};

export class OrchestratorClient {
  constructor(private readonly config: GatewayConfig) {}

  async run(input: {
    goal: string;
    sessionId: string;
    userId?: string;
    authContext?: OrchestratorAuthContext;
    signal?: AbortSignal;
  }) {
    const timeoutController = new AbortController();
    const timeout = setTimeout(() => timeoutController.abort(), this.config.orchestratorTimeoutMs);
    const signal = combineSignals([timeoutController.signal, input.signal]);

    try {
      const response = await fetch(`${this.config.orchestratorBaseUrl}/agent/run`, {
        method: "POST",
        headers: {
          "Content-Type": "application/json",
        },
        body: JSON.stringify({
          goal: input.goal,
          session_id: input.sessionId,
          user_id: input.userId,
          auth_context: input.authContext
            ? {
                user_id: input.authContext.userId,
                verified: input.authContext.verified,
                scope: input.authContext.scope,
                verified_by: input.authContext.verifiedBy,
                verified_at: input.authContext.verifiedAt,
              }
            : undefined,
        }),
        signal,
      });

      if (!response.ok) {
        const body = await safeReadJson(response);
        throw new GatewayError(
          "ORCHESTRATOR_FAILED",
          readErrorMessage(body) ?? `Orchestrator failed with status ${response.status}.`,
          502,
        );
      }

      const body = (await response.json()) as OrchestratorResponse;
      if (body.ok === false || body.success === false) {
        throw new GatewayError(
          "ORCHESTRATOR_FAILED",
          body.error?.message || body.message || "Orchestrator returned an unsuccessful response.",
          502,
        );
      }

      return body;
    } catch (error) {
      if (error instanceof GatewayError) {
        throw error;
      }

      if (timeoutController.signal.aborted) {
        throw new GatewayError("ORCHESTRATOR_TIMEOUT", "Orchestrator request timed out.", 504);
      }

      if (error instanceof Error && error.name === "AbortError") {
        throw new GatewayError("ORCHESTRATOR_FAILED", "Orchestrator request was cancelled.", 499);
      }

      throw new GatewayError("ORCHESTRATOR_UNAVAILABLE", "Could not reach Orchestrator service.", 502);
    } finally {
      clearTimeout(timeout);
    }
  }

  async resume(input: {
    sessionId: string;
    userId?: string;
    authContext?: OrchestratorAuthContext;
    approval: {
      stepId: string;
      decision: "approve" | "reject" | "revise";
      comment?: string;
    };
    signal?: AbortSignal;
  }) {
    const timeoutController = new AbortController();
    const timeout = setTimeout(() => timeoutController.abort(), this.config.orchestratorTimeoutMs);
    const signal = combineSignals([timeoutController.signal, input.signal]);

    try {
      const response = await fetch(`${this.config.orchestratorBaseUrl}/agent/resume`, {
        method: "POST",
        headers: {
          "Content-Type": "application/json",
        },
        body: JSON.stringify({
          session_id: input.sessionId,
          user_id: input.userId,
          auth_context: input.authContext
            ? {
                user_id: input.authContext.userId,
                verified: input.authContext.verified,
                scope: input.authContext.scope,
                verified_by: input.authContext.verifiedBy,
                verified_at: input.authContext.verifiedAt,
              }
            : undefined,
          approval: {
            step_id: input.approval.stepId,
            decision: input.approval.decision,
            comment: input.approval.comment,
          },
        }),
        signal,
      });

      if (!response.ok) {
        const body = await safeReadJson(response);
        throw new GatewayError(
          "ORCHESTRATOR_FAILED",
          readErrorMessage(body) ?? `Orchestrator resume failed with status ${response.status}.`,
          502,
        );
      }

      const body = (await response.json()) as OrchestratorResponse;
      if (body.ok === false || body.success === false) {
        throw new GatewayError(
          "ORCHESTRATOR_FAILED",
          body.error?.message || body.message || "Orchestrator returned an unsuccessful resume response.",
          502,
        );
      }

      return body;
    } catch (error) {
      if (error instanceof GatewayError) {
        throw error;
      }

      if (timeoutController.signal.aborted) {
        throw new GatewayError("ORCHESTRATOR_TIMEOUT", "Orchestrator resume request timed out.", 504);
      }

      throw new GatewayError("ORCHESTRATOR_UNAVAILABLE", "Could not reach Orchestrator service.", 502);
    } finally {
      clearTimeout(timeout);
    }
  }
}

function combineSignals(signals: Array<AbortSignal | undefined>) {
  const controller = new AbortController();
  for (const signal of signals) {
    if (!signal) {
      continue;
    }
    if (signal.aborted) {
      controller.abort();
      break;
    }
    signal.addEventListener("abort", () => controller.abort(), { once: true });
  }
  return controller.signal;
}

async function safeReadJson(response: Response) {
  try {
    return await response.json();
  } catch {
    return null;
  }
}

function readErrorMessage(value: unknown) {
  if (
    typeof value === "object" &&
    value &&
    "error" in value &&
    typeof value.error === "object" &&
    value.error &&
    "message" in value.error &&
    typeof value.error.message === "string"
  ) {
    return value.error.message;
  }
  if (typeof value === "object" && value && "message" in value && typeof value.message === "string") {
    return value.message;
  }
  return undefined;
}
