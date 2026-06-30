import { Router } from "express";
import type { GatewayConfig } from "../config.js";
import { GatewayError } from "../services/errors.js";
import { sessionStore } from "../services/sessionStore.js";
import { requireSessionAccess } from "../services/sessionAccess.js";
import type { RunProcessor } from "../services/runProcessor.js";
import { asyncHandler } from "../middleware/asyncHandler.js";
import { routeParam } from "./params.js";

export function runsRouter(runProcessor: RunProcessor, config: GatewayConfig) {
  const router = Router();

  router.post(
    "/sessions/:sessionId/runs/:runId/cancel",
    asyncHandler(async (request, response) => {
      const sessionId = routeParam(request.params.sessionId, "sessionId");
      const runId = routeParam(request.params.runId, "runId");
      requireSessionAccess(request, config, sessionId);

      const run = runProcessor.cancel(sessionId, runId);
      if (!run) {
        throw new GatewayError("RUN_NOT_FOUND", "Run not found.", 404);
      }

      response.json({
        accepted: true,
        run,
      });
    }),
  );

  return router;
}
