import type { Request } from "express";
import type { GatewayConfig } from "../config.js";
import { buildAuthContext } from "./authContext.js";
import { GatewayError } from "./errors.js";
import { sessionStore } from "./sessionStore.js";

export function requireSessionAccess(request: Request, config: GatewayConfig, sessionId: string) {
  const session = sessionStore.getSession(sessionId);
  if (!session) {
    throw new GatewayError("SESSION_NOT_FOUND", "Session not found.", 404);
  }

  const authContext = buildAuthContext(request, config, session.userId);
  if (session.userId && !authContext?.userId) {
    throw new GatewayError("SESSION_NOT_FOUND", "Session not found.", 404);
  }
  if (session.userId && authContext?.userId && session.userId !== authContext.userId) {
    throw new GatewayError("SESSION_NOT_FOUND", "Session not found.", 404);
  }

  return { session, authContext };
}
