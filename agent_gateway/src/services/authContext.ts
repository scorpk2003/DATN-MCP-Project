import crypto from "node:crypto";
import type { Request } from "express";
import type { GatewayConfig } from "../config.js";

export type TrustedAuthContext = {
  userId: string;
  verified: boolean;
  scope: string[];
  verifiedBy: string;
  verifiedAt: string;
};

const DEFAULT_SCOPES = [
  "roadmap:read",
  "roadmap:write",
  "lesson:write",
  "lesson:evaluate",
  "lesson:progress",
];

export function buildAuthContext(
  request: Request | undefined,
  config: GatewayConfig,
  fallbackUserId?: string,
): TrustedAuthContext | undefined {
  const bearer = readBearerToken(request);
  const explicitUserId = readHeader(request, "x-user-id") ?? fallbackUserId;

  if (bearer) {
    return {
      userId: explicitUserId ?? `firebase:${stableHash(bearer)}`,
      verified: true,
      scope: DEFAULT_SCOPES,
      verifiedBy: "agent_gateway_bearer",
      verifiedAt: new Date().toISOString(),
    };
  }

  if (!config.allowDevAuthContext && !explicitUserId) {
    return undefined;
  }

  return {
    userId: explicitUserId ?? "dev-user",
    verified: config.allowDevAuthContext,
    scope: DEFAULT_SCOPES,
    verifiedBy: config.allowDevAuthContext ? "agent_gateway_dev" : "agent_gateway_unverified",
    verifiedAt: new Date().toISOString(),
  };
}

function readBearerToken(request: Request | undefined) {
  const header = request?.headers.authorization;
  if (!header) {
    return undefined;
  }

  const value = Array.isArray(header) ? header[0] : header;
  const token = value.trim().match(/^Bearer\s+(.+)$/i)?.[1]?.trim();
  return token || undefined;
}

function readHeader(request: Request | undefined, key: string) {
  const value = request?.headers[key];
  if (Array.isArray(value)) {
    return value[0];
  }
  return typeof value === "string" && value.trim() ? value.trim() : undefined;
}

function stableHash(value: string) {
  return crypto.createHash("sha256").update(value).digest("hex").slice(0, 24);
}
