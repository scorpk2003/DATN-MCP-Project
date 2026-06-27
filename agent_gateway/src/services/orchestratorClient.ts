import type { GatewayConfig } from "../config.js";
import { GatewayError } from "./errors.js";

export type OrchestratorResponse = {
  success: boolean;
  output?: unknown;
  message: string;
};

export class OrchestratorClient {
  constructor(private readonly config: GatewayConfig) {}

  async run(input: { goal: string; sessionId: string; signal?: AbortSignal }) {
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
      if (!body.success) {
        throw new GatewayError("ORCHESTRATOR_FAILED", body.message || "Orchestrator returned an unsuccessful response.", 502);
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
  if (typeof value === "object" && value && "message" in value && typeof value.message === "string") {
    return value.message;
  }
  return undefined;
}
