import { Router } from "express";
import type { GatewayConfig } from "../config.js";
import { GatewayError } from "../services/errors.js";
import { eventBus } from "../services/eventBus.js";
import { sessionStore } from "../services/sessionStore.js";
import { requireSessionAccess } from "../services/sessionAccess.js";
import { asyncHandler } from "../middleware/asyncHandler.js";
import { routeParam } from "./params.js";

const TERMINAL_STATUSES = new Set(["completed", "failed", "cancelled"]);

export function streamRouter(config: GatewayConfig) {
  const router = Router();

  router.get(
    "/sessions/:sessionId/runs/:runId/stream",
    asyncHandler(async (request, response) => {
      const sessionId = routeParam(request.params.sessionId, "sessionId");
      const runId = routeParam(request.params.runId, "runId");
      requireSessionAccess(request, config, sessionId);
      const run = sessionStore.getRun(sessionId, runId);
      if (!run) {
        throw new GatewayError("RUN_NOT_FOUND", "Run not found.", 404);
      }

      response.setHeader("Content-Type", "text/event-stream");
      response.setHeader("Cache-Control", "no-cache, no-transform");
      response.setHeader("Connection", "keep-alive");
      response.flushHeaders?.();

      const writeEnvelope = (envelope: unknown) => {
        response.write("event: agent_event\n");
        response.write(`data: ${JSON.stringify(envelope)}\n\n`);
      };

      for (const envelope of sessionStore.getRunEnvelopes(sessionId, runId)) {
        writeEnvelope(envelope);
      }

      const unsubscribe = eventBus.subscribe(sessionId, runId, (envelope) => {
        writeEnvelope(envelope);
        if (envelope.event.type === "run.status_changed" && TERMINAL_STATUSES.has(envelope.event.status)) {
          unsubscribe();
          response.end();
        }
      });

      const latestRun = sessionStore.getRun(sessionId, runId);
      if (latestRun && TERMINAL_STATUSES.has(latestRun.status)) {
        unsubscribe();
        response.end();
      }

      request.on("close", () => {
        unsubscribe();
      });
    }),
  );

  return router;
}
