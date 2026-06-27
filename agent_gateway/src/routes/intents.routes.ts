import { Router } from "express";
import { sendIntentRequestSchema } from "../protocol/index.js";
import { intentToUserMessage } from "../adapters/intentAdapter.js";
import { GatewayError } from "../services/errors.js";
import { eventBus } from "../services/eventBus.js";
import { sessionStore } from "../services/sessionStore.js";
import type { RunProcessor } from "../services/runProcessor.js";
import { asyncHandler } from "../middleware/asyncHandler.js";
import { routeParam } from "./params.js";

export function intentsRouter(runProcessor: RunProcessor) {
  const router = Router();

  router.post(
    "/sessions/:sessionId/intents",
    asyncHandler(async (request, response) => {
      const sessionId = routeParam(request.params.sessionId, "sessionId");
      const session = sessionStore.getSession(sessionId);
      if (!session) {
        throw new GatewayError("SESSION_NOT_FOUND", "Session not found.", 404);
      }

      const input = sendIntentRequestSchema.safeParse(request.body);
      if (!input.success) {
        throw new GatewayError("INVALID_INTENT", input.error.issues[0]?.message ?? "Invalid user intent.", 400);
      }
      const run = sessionStore.createRun(sessionId);
      if (!run) {
        throw new GatewayError("SESSION_NOT_FOUND", "Session not found.", 404);
      }

      sessionStore.addUserMessage(sessionId, run.id, intentToUserMessage(input.data.intent));
      eventBus.publish(sessionId, run.id, {
        type: "run.status_changed",
        status: "queued",
      });
      runProcessor.start({
        sessionId,
        runId: run.id,
        intent: input.data.intent,
      });

      response.status(202).json({
        accepted: true,
        run,
        streamUrl: `/sessions/${sessionId}/runs/${run.id}/stream`,
      });
    }),
  );

  return router;
}
