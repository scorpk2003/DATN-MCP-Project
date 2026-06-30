import { Router } from "express";
import { z } from "zod";
import { createSessionRequestSchema } from "../protocol/index.js";
import type { GatewayConfig } from "../config.js";
import { buildAuthContext } from "../services/authContext.js";
import { GatewayError } from "../services/errors.js";
import { sessionStore } from "../services/sessionStore.js";
import { requireSessionAccess } from "../services/sessionAccess.js";
import { asyncHandler } from "../middleware/asyncHandler.js";
import { routeParam } from "./params.js";

export function sessionsRouter(config: GatewayConfig) {
  const router = Router();

  router.post(
    "/sessions",
    asyncHandler(async (request, response) => {
      const input = createSessionRequestSchema.parse(request.body ?? {});
      const authContext = buildAuthContext(request, config, input.userId);
      const session = sessionStore.createSession({
        ...input,
        userId: authContext?.userId,
      });
      response.status(201).json({ session });
    }),
  );

  router.get(
    "/sessions",
    asyncHandler(async (request, response) => {
      const query = z
        .object({
          userId: z.string().optional(),
          limit: z.coerce.number().int().positive().max(100).optional(),
        })
        .parse(request.query);
      const authContext = buildAuthContext(request, config);
      response.json({
        sessions: sessionStore.listSessions({
          userId: authContext?.userId,
          limit: query.limit,
        }),
      });
    }),
  );

  router.get(
    "/sessions/:sessionId/state",
    asyncHandler(async (request, response) => {
      const sessionId = routeParam(request.params.sessionId, "sessionId");
      requireSessionAccess(request, config, sessionId);
      const state = sessionStore.getState(sessionId);
      if (!state) {
        throw new GatewayError("SESSION_NOT_FOUND", "Session not found.", 404);
      }

      response.json(state);
    }),
  );

  return router;
}
