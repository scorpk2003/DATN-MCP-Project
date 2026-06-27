import "dotenv/config";

export type GatewayConfig = {
  host: string;
  port: number;
  orchestratorBaseUrl: string;
  orchestratorTimeoutMs: number;
  corsOrigin: string;
};

function readNumber(name: string, fallback: number) {
  const raw = process.env[name];
  if (!raw) {
    return fallback;
  }

  const parsed = Number(raw);
  return Number.isFinite(parsed) ? parsed : fallback;
}

export const config: GatewayConfig = {
  host: process.env.AGENT_GATEWAY_HOST || "0.0.0.0",
  port: readNumber("AGENT_GATEWAY_PORT", 4000),
  orchestratorBaseUrl: process.env.ORCHESTRATOR_BASE_URL || "http://localhost:3000",
  orchestratorTimeoutMs: readNumber("ORCHESTRATOR_TIMEOUT_MS", 120000),
  corsOrigin: process.env.CORS_ORIGIN || "http://localhost:5173",
};
