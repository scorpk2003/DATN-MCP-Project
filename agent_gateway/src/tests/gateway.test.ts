import assert from "node:assert/strict";
import { once } from "node:events";
import { describe, it } from "node:test";
import type { Server } from "node:http";
import { createApp } from "../app.js";
import type { GatewayConfig } from "../config.js";

const testConfig: GatewayConfig = {
  host: "127.0.0.1",
  port: 0,
  orchestratorBaseUrl: "http://127.0.0.1:9",
  orchestratorTimeoutMs: 200,
  corsOrigin: "*",
  resourceServiceBaseUrl: "http://127.0.0.1:9",
  allowDevAuthContext: true,
};

async function withServer<T>(callback: (baseUrl: string) => Promise<T>) {
  const server = createApp(testConfig).listen(0, "127.0.0.1");
  await once(server, "listening");
  const address = server.address();
  assert(address && typeof address === "object");
  const baseUrl = `http://127.0.0.1:${address.port}`;

  try {
    return await callback(baseUrl);
  } finally {
    await closeServer(server);
  }
}

describe("agent gateway", () => {
  it("returns health status", async () => {
    await withServer(async (baseUrl) => {
      const response = await fetch(`${baseUrl}/health`);
      assert.equal(response.status, 200);
      assert.deepEqual(await response.json(), { status: "ok" });
    });
  });

  it("creates a session and returns state", async () => {
    await withServer(async (baseUrl) => {
      const createResponse = await fetch(`${baseUrl}/sessions`, {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({ title: "Learn MCP" }),
      });
      assert.equal(createResponse.status, 201);
      const created = (await createResponse.json()) as { session: { id: string } };

      const stateResponse = await fetch(`${baseUrl}/sessions/${created.session.id}/state`);
      assert.equal(stateResponse.status, 200);
      const state = (await stateResponse.json()) as { session: { id: string }; messages: unknown[] };
      assert.equal(state.session.id, created.session.id);
      assert.deepEqual(state.messages, []);
    });
  });

  it("rejects invalid intents", async () => {
    await withServer(async (baseUrl) => {
      const session = await createSession(baseUrl);
      const response = await fetch(`${baseUrl}/sessions/${session.id}/intents`, {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({ intent: { type: "goal.submitted", payload: { goal: "" } } }),
      });

      assert.equal(response.status, 400);
      const body = (await response.json()) as { error: { code: string } };
      assert.equal(body.error.code, "INVALID_INTENT");
    });
  });

  it("accepts a valid intent and exposes SSE history", async () => {
    await withServer(async (baseUrl) => {
      const session = await createSession(baseUrl);
      const intentResponse = await fetch(`${baseUrl}/sessions/${session.id}/intents`, {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({
          intent: {
            type: "goal.submitted",
            payload: { goal: "Learn Rust MCP" },
          },
        }),
      });
      assert.equal(intentResponse.status, 202);
      const accepted = (await intentResponse.json()) as { run: { id: string }; streamUrl: string };

      const streamResponse = await fetch(`${baseUrl}${accepted.streamUrl}`);
      assert.equal(streamResponse.status, 200);
      const firstChunk = await readStreamChunk(streamResponse);
      assert.match(firstChunk, /event: agent_event/);
      assert.match(firstChunk, /run.status_changed/);

      await waitForRunTerminal(baseUrl, session.id, accepted.run.id);
      const stateResponse = await fetch(`${baseUrl}/sessions/${session.id}/state`);
      const state = (await stateResponse.json()) as { messages: unknown[] };
      assert.ok(state.messages.length >= 1);
    });
  });
});

async function createSession(baseUrl: string) {
  const response = await fetch(`${baseUrl}/sessions`, {
    method: "POST",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify({ title: "Test session" }),
  });
  const body = (await response.json()) as { session: { id: string } };
  return body.session;
}

async function readStreamChunk(response: Response) {
  const reader = response.body?.getReader();
  assert(reader);
  const { value } = await reader.read();
  reader.cancel();
  return new TextDecoder().decode(value);
}

async function waitForRunTerminal(baseUrl: string, sessionId: string, runId: string) {
  for (let attempt = 0; attempt < 20; attempt += 1) {
    const response = await fetch(`${baseUrl}/sessions/${sessionId}/state`);
    const state = (await response.json()) as { activeRun?: { id: string } };
    if (!state.activeRun || state.activeRun.id !== runId) {
      return;
    }
    await new Promise((resolve) => setTimeout(resolve, 50));
  }
}

function closeServer(server: Server) {
  return new Promise<void>((resolve, reject) => {
    server.close((error) => {
      if (error) {
        reject(error);
        return;
      }
      resolve();
    });
  });
}
