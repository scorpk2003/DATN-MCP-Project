import { Router } from "express";
import { respondActionRequestSchema, type UserIntent } from "../protocol/index.js";
import { GatewayError } from "../services/errors.js";
import { eventBus } from "../services/eventBus.js";
import { sessionStore } from "../services/sessionStore.js";
import type { RunProcessor } from "../services/runProcessor.js";
import { asyncHandler } from "../middleware/asyncHandler.js";
import { routeParam } from "./params.js";

export function actionsRouter(runProcessor: RunProcessor) {
  const router = Router();

  router.post(
    "/sessions/:sessionId/actions/:actionId/respond",
    asyncHandler(async (request, response) => {
      const sessionId = routeParam(request.params.sessionId, "sessionId");
      const actionId = routeParam(request.params.actionId, "actionId");
      if (!sessionStore.getSession(sessionId)) {
        throw new GatewayError("SESSION_NOT_FOUND", "Session not found.", 404);
      }

      const action = sessionStore.getAction(sessionId, actionId);
      if (!action) {
        throw new GatewayError("ACTION_NOT_FOUND", "Action not found.", 404);
      }
      if (action.status === "resolved") {
        throw new GatewayError("ACTION_ALREADY_RESOLVED", "Action is already resolved.", 409);
      }

      const input = respondActionRequestSchema.parse(request.body);
      if (!action.options.some((option) => option.id === input.selectedOptionId)) {
        throw new GatewayError("INVALID_REQUEST", "Selected option is not available for this action.", 400);
      }

      const resolved = sessionStore.resolveAction(sessionId, actionId);
      if (!resolved) {
        throw new GatewayError("ACTION_ALREADY_RESOLVED", "Action is already resolved.", 409);
      }

      const run = sessionStore.createRun(sessionId, action.runId);
      if (!run) {
        throw new GatewayError("SESSION_NOT_FOUND", "Session not found.", 404);
      }

      const intent = actionResponseToIntent(input.selectedOptionId, input.payload);
      eventBus.publish(sessionId, run.id, {
        type: "run.status_changed",
        status: "queued",
      });
      runProcessor.start({
        sessionId,
        runId: run.id,
        intent,
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

function actionResponseToIntent(selectedOptionId: string, payload: Record<string, unknown> | undefined): UserIntent {
  if (selectedOptionId === "backfill_first") {
    return {
      type: "resource.backfill.requested",
      payload: {
        topicId: typeof payload?.topicId === "string" ? payload.topicId : "current_roadmap",
        reason: "User requested resource backfill from a pending action.",
        priority: "normal",
      },
    };
  }

  return {
    type: "chat.submitted",
    payload: {
      message: `User selected action option: ${selectedOptionId}`,
    },
  };
}
