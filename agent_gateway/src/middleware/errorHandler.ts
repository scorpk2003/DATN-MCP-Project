import type { ErrorRequestHandler } from "express";
import { ZodError } from "zod";
import { GatewayError, errorBody } from "../services/errors.js";

export const errorHandler: ErrorRequestHandler = (error, _request, response, _next) => {
  if (error instanceof GatewayError) {
    response.status(error.status).json(errorBody(error.code, error.message));
    return;
  }

  if (error instanceof ZodError) {
    response.status(400).json(errorBody("INVALID_REQUEST", error.issues[0]?.message ?? "Invalid request."));
    return;
  }

  response.status(500).json(errorBody("INTERNAL_ERROR", "Unexpected gateway error."));
};
