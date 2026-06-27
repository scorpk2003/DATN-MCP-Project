export type GatewayErrorCode =
  | "INVALID_INTENT"
  | "INVALID_REQUEST"
  | "SESSION_NOT_FOUND"
  | "RUN_NOT_FOUND"
  | "ACTION_NOT_FOUND"
  | "ACTION_ALREADY_RESOLVED"
  | "ORCHESTRATOR_TIMEOUT"
  | "ORCHESTRATOR_UNAVAILABLE"
  | "ORCHESTRATOR_FAILED"
  | "ARTIFACT_VALIDATION_FAILED";

export class GatewayError extends Error {
  constructor(
    public readonly code: GatewayErrorCode,
    message: string,
    public readonly status = 500,
  ) {
    super(message);
  }
}

export function errorBody(code: string, message: string) {
  return {
    error: {
      code,
      message,
    },
  };
}
