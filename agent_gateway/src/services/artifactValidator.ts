import type { UIArtifact } from "../protocol/index.js";
import { uiArtifactSchema } from "../protocol/index.js";
import { GatewayError } from "./errors.js";

export function validateArtifact(artifact: UIArtifact) {
  const parsed = uiArtifactSchema.safeParse(artifact);
  if (!parsed.success) {
    throw new GatewayError("ARTIFACT_VALIDATION_FAILED", "Agent output did not match a supported UI artifact.", 422);
  }

  return parsed.data;
}
