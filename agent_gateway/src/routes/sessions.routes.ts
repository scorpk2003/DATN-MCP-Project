import { Router } from "express";
import { z } from "zod";
import { createSessionRequestSchema } from "../protocol/index.js";
import { GatewayError } from "../services/errors.js";
import { sessionStore } from "../services/sessionStore.js";
import { asyncHandler } from "../middleware/asyncHandler.js";
import { routeParam } from "./params.js";

export function sessionsRouter() {
  const router = Router();

  router.post(
    "/sessions",
    asyncHandler(async (request, response) => {
      const input = createSessionRequestSchema.parse(request.body ?? {});
      const session = sessionStore.createSession(input);
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
      response.json({
        sessions: sessionStore.listSessions(query),
      });
    }),
  );

  router.get(
    "/sessions/:sessionId/state",
    asyncHandler(async (request, response) => {
      const sessionId = routeParam(request.params.sessionId, "sessionId");
      const state = sessionStore.getState(sessionId);
      if (!state) {
        throw new GatewayError("SESSION_NOT_FOUND", "Session not found.", 404);
      }

      response.json(state);
    }),
  );

  return router;
}
