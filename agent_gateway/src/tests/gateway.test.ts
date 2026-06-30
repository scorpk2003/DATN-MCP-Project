import assert from "node:assert/strict";
import { once } from "node:events";
import { describe, it } from "node:test";
import { createServer, type IncomingMessage, type Server } from "node:http";
import { createApp } from "../app.js";
import type { GatewayConfig } from "../config.js";
import { intentToGoal, intentToOrchestratorContext } from "../adapters/intentAdapter.js";
import { sessionStore } from "../services/sessionStore.js";

const testConfig: GatewayConfig = {
  host: "127.0.0.1",
  port: 0,
  orchestratorBaseUrl: "http://127.0.0.1:9",
  orchestratorTimeoutMs: 200,
  corsOrigin: "*",
  resourceServiceBaseUrl: "http://127.0.0.1:9",
  databaseMcpBaseUrl: "http://127.0.0.1:9",
  allowDevAuthContext: true,
};

async function withServer<T>(callback: (baseUrl: string) => Promise<T>, config: GatewayConfig = testConfig) {
  sessionStore.clearForTest();
  const server = createApp(config).listen(0, "127.0.0.1");
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

  it("uses real sessions for navigation recent chats", async () => {
    await withServer(async (baseUrl) => {
      const session = await createSession(baseUrl, "Navigation session");

      const response = await fetch(`${baseUrl}/navigation`);
      assert.equal(response.status, 200);
      const body = (await response.json()) as { recentChats: Array<{ id: string; label: string; path: string }> };
      assert.ok(body.recentChats.some((chat) => chat.id === session.id && chat.label === "Navigation session" && chat.path === "/"));
    });
  });

  it("scopes navigation and session state by authenticated user", async () => {
    await withServer(async (baseUrl) => {
      const userA = await createSession(baseUrl, "User A session", { "X-User-ID": "user-a" });
      const userB = await createSession(baseUrl, "User B session", { "X-User-ID": "user-b" });

      const navigationResponse = await fetch(`${baseUrl}/navigation`, {
        headers: { "X-User-ID": "user-a" },
      });
      assert.equal(navigationResponse.status, 200);
      const navigation = (await navigationResponse.json()) as { recentChats: Array<{ id: string; label: string }> };
      assert.ok(navigation.recentChats.some((chat) => chat.id === userA.id && chat.label === "User A session"));
      assert.ok(!navigation.recentChats.some((chat) => chat.id === userB.id));

      const deniedState = await fetch(`${baseUrl}/sessions/${userB.id}/state`, {
        headers: { "X-User-ID": "user-a" },
      });
      assert.equal(deniedState.status, 404);
    });
  });

  it("returns an empty roadmap page contract when no roadmap artifact exists", async () => {
    await withServer(async (baseUrl) => {
      const response = await fetch(`${baseUrl}/roadmap`);
      assert.equal(response.status, 200);
      assert.deepEqual(await response.json(), {
        roadmapSummary: null,
        roadmapPhases: [],
        studyResources: [],
      });
    });
  });

  it("unwraps resource service page envelopes without fallback resources", async () => {
    const resourceServer = createServer((_request, response) => {
      response.setHeader("Content-Type", "application/json");
      response.end(
        JSON.stringify({
          success: true,
          data: {
            items: [
              {
                resourceId: "resource-real",
                canonicalUrl: "https://example.test/postgres-indexes",
                title: "Seeded PostgreSQL indexing guide",
                summary: "A real seeded resource.",
                resourceType: "official_docs",
                updatedAt: "2026-06-30",
              },
            ],
            pagination: { limit: 20, offset: 0, total: 1, hasMore: false },
          },
          error: null,
          meta: { requestId: "req_test", timestamp: "0" },
        }),
      );
    });
    resourceServer.listen(0, "127.0.0.1");
    await once(resourceServer, "listening");
    const address = resourceServer.address();
    assert(address && typeof address === "object");

    try {
      await withServer(
        async (baseUrl) => {
          const response = await fetch(`${baseUrl}/resources`);
          assert.equal(response.status, 200);
          const body = (await response.json()) as { resources: Array<{ id: string; title: string; url?: string }> };
          assert.equal(body.resources.length, 1);
          assert.equal(body.resources[0]?.id, "resource-real");
          assert.equal(body.resources[0]?.title, "Seeded PostgreSQL indexing guide");
          assert.equal(body.resources[0]?.url, "https://example.test/postgres-indexes");
        },
        {
          ...testConfig,
          resourceServiceBaseUrl: `http://127.0.0.1:${address.port}`,
        },
      );
    } finally {
      await closeServer(resourceServer);
    }
  });

  it("returns an empty resource page contract when resource service fails", async () => {
    const resourceServer = createServer((_request, response) => {
      response.statusCode = 500;
      response.end("resource service unavailable");
    });
    resourceServer.listen(0, "127.0.0.1");
    await once(resourceServer, "listening");
    const address = resourceServer.address();
    assert(address && typeof address === "object");

    try {
      await withServer(
        async (baseUrl) => {
          const response = await fetch(`${baseUrl}/resources`);
          assert.equal(response.status, 200);
          assert.deepEqual(await response.json(), {
            resources: [],
            resourceCourses: ["Tất cả"],
            resourceTypes: ["Tất cả"],
          });
        },
        {
          ...testConfig,
          resourceServiceBaseUrl: `http://127.0.0.1:${address.port}`,
        },
      );
    } finally {
      await closeServer(resourceServer);
    }
  });

  it("creates notes through the database mcp rest endpoint", async () => {
    let capturedBody: Record<string, unknown> | null = null;
    const databaseServer = createServer(async (request, response) => {
      assert.equal(request.method, "POST");
      assert.equal(request.url, "/notes");
      capturedBody = JSON.parse(await readRequestBody(request));
      response.setHeader("Content-Type", "application/json");
      response.statusCode = 201;
      response.end(
        JSON.stringify({
          note: {
            id: "note-db",
            content: "New note title\nDetails",
            created_at: "2026-06-30T00:00:00Z",
          },
        }),
      );
    });
    databaseServer.listen(0, "127.0.0.1");
    await once(databaseServer, "listening");
    const address = databaseServer.address();
    assert(address && typeof address === "object");

    try {
      await withServer(
        async (baseUrl) => {
          const response = await fetch(`${baseUrl}/notes`, {
            method: "POST",
            headers: { "Content-Type": "application/json", "X-User-ID": "dev-learner" },
            body: JSON.stringify({ content: "New note title\nDetails" }),
          });

          assert.equal(response.status, 201);
          const body = (await response.json()) as { note: { id: string; title: string } };
          assert.equal(body.note.id, "note-db");
          assert.equal(body.note.title, "New note title");
          assert.deepEqual(capturedBody, {
            userId: "dev-learner",
            content: "New note title\nDetails",
          });
        },
        {
          ...testConfig,
          databaseMcpBaseUrl: `http://127.0.0.1:${address.port}`,
        },
      );
    } finally {
      await closeServer(databaseServer);
    }
  });

  it("uses persisted database roadmap data for the roadmap page contract", async () => {
    const databaseServer = createServer((request, response) => {
      assert.equal(request.url, "/roadmaps/latest?userId=dev-user");
      response.setHeader("Content-Type", "application/json");
      response.end(
        JSON.stringify({
          roadmap: {
            id: "roadmap-db",
            title: "Persisted PostgreSQL roadmap",
            project_description: "Saved through Database MCP.",
            generated_by: "agent",
            phases: [
              {
                id: "phase-db",
                title: "Index foundations",
                estimated_days: 3,
                milestones: [
                  {
                    id: "milestone-db",
                    title: "B-tree basics",
                    tasks: [
                      {
                        id: "task-db",
                        title: "Explain B-tree lookup",
                        difficulty: "medium",
                        status: "pending",
                      },
                    ],
                  },
                ],
              },
            ],
          },
        }),
      );
    });
    databaseServer.listen(0, "127.0.0.1");
    await once(databaseServer, "listening");
    const address = databaseServer.address();
    assert(address && typeof address === "object");

    try {
      await withServer(
        async (baseUrl) => {
          const response = await fetch(`${baseUrl}/roadmap`);
          assert.equal(response.status, 200);
          const body = (await response.json()) as {
            roadmapSummary: { title: string };
            roadmapPhases: Array<{
              title: string;
              tasks: Array<{ id: string; taskId: string; title: string; status: string }>;
            }>;
          };
          assert.equal(body.roadmapSummary.title, "Persisted PostgreSQL roadmap");
          assert.equal(body.roadmapPhases[0]?.title, "Index foundations");
          assert.deepEqual(body.roadmapPhases[0]?.tasks, [
            {
              id: "task-db",
              taskId: "task-db",
              title: "Explain B-tree lookup",
              status: "pending",
              difficulty: "medium",
              level: "intermediate",
            },
          ]);
        },
        {
          ...testConfig,
          databaseMcpBaseUrl: `http://127.0.0.1:${address.port}`,
        },
      );
    } finally {
      await closeServer(databaseServer);
    }
  });

  it("maps the latest roadmap artifact into the roadmap page contract", async () => {
    await withServer(async (baseUrl) => {
      const session = await createSession(baseUrl, "Artifact session");
      const run = sessionStore.createRun(session.id);
      assert.ok(run);
      sessionStore.appendEnvelope(session.id, run.id, {
        type: "artifact.created",
        artifact: {
          kind: "roadmap",
          id: "roadmap-test",
          title: "PostgreSQL indexing roadmap",
          goal: "Learn PostgreSQL indexes",
          status: "active",
          coverageStatus: "good",
          nodes: [
            {
              id: "node-btree",
              title: "B-tree indexes",
              type: "concept",
              status: "active",
              coverageStatus: "good",
            },
          ],
          edges: [],
        },
      });

      const response = await fetch(`${baseUrl}/roadmap`);
      assert.equal(response.status, 200);
      const body = (await response.json()) as { roadmapSummary: { title: string }; roadmapPhases: Array<{ title: string }> };
      assert.equal(body.roadmapSummary.title, "PostgreSQL indexing roadmap");
      assert.equal(body.roadmapPhases[0]?.title, "B-tree indexes");
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

  it("accepts roadmap task learning intents from page actions", async () => {
    await withServer(async (baseUrl) => {
      const session = await createSession(baseUrl);
      const intentResponse = await fetch(`${baseUrl}/sessions/${session.id}/intents`, {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({
          intent: {
            type: "roadmap.task.selected",
            payload: {
              roadmapId: "roadmap-db",
              phaseId: "phase-db",
              taskId: "task-db",
              title: "Database task appears on roadmap",
            },
          },
        }),
      });

      assert.equal(intentResponse.status, 202);
    });
  });

  it("keeps roadmap task lesson level in orchestrator goal and context", () => {
    const intent = {
      type: "roadmap.task.selected" as const,
      payload: {
        roadmapId: "roadmap-db",
        phaseId: "phase-db",
        taskId: "task-db",
        title: "Database task appears on roadmap",
        level: "intermediate" as const,
      },
    };

    assert.match(intentToGoal(intent), /Lesson level: intermediate/);
    assert.deepEqual(intentToOrchestratorContext(intent), {
      type: "roadmap.task.selected",
      roadmapId: "roadmap-db",
      phaseId: "phase-db",
      milestoneId: undefined,
      taskId: "task-db",
      nodeId: "task-db",
      roadmapNodeId: "task-db",
      title: "Database task appears on roadmap",
      description: undefined,
      level: "intermediate",
    });
  });

  it("preserves review intent context across approval resume for lesson artifacts", async () => {
    const orchestrator = createServer(async (request, response) => {
      response.setHeader("Content-Type", "application/json");
      if (request.url === "/agent/run") {
        response.end(
          JSON.stringify({
            ok: true,
            status: "waiting_for_user",
            output: {
              approval: {
                step_id: "approve-review-lesson",
                question: "Approve review lesson generation.",
              },
            },
          }),
        );
        return;
      }

      if (request.url === "/agent/resume") {
        response.end(
          JSON.stringify({
            ok: true,
            status: "completed",
            output: {
              ok: true,
              status: "completed",
              output: {},
            },
          }),
        );
        return;
      }

      response.statusCode = 404;
      response.end(JSON.stringify({ ok: false }));
    });
    orchestrator.listen(0, "127.0.0.1");
    await once(orchestrator, "listening");
    const address = orchestrator.address();
    assert(address && typeof address === "object");

    try {
      await withServer(
        async (baseUrl) => {
          const session = await createSession(baseUrl);
          const intentResponse = await fetch(`${baseUrl}/sessions/${session.id}/intents`, {
            method: "POST",
            headers: { "Content-Type": "application/json" },
            body: JSON.stringify({
              intent: {
                type: "review.task.selected",
                payload: {
                  taskId: "task-core-1",
                  concept: "Core concept 1",
                  course: "Seeded course",
                  confidence: 0.35,
                  due: "Today",
                },
              },
            }),
          });
          assert.equal(intentResponse.status, 202);
          const accepted = (await intentResponse.json()) as { run: { id: string } };

          const waitingState = await waitForRunStatus(baseUrl, session.id, accepted.run.id, "waiting_for_user");
          assert.equal(waitingState.pendingActions.length, 1);
          const action = waitingState.pendingActions[0];
          assert.ok(action);

          const approveResponse = await fetch(`${baseUrl}/sessions/${session.id}/actions/${action.id}/respond`, {
            method: "POST",
            headers: { "Content-Type": "application/json" },
            body: JSON.stringify({ selectedOptionId: "approve" }),
          });
          assert.equal(approveResponse.status, 202);
          const approved = (await approveResponse.json()) as { run: { id: string } };

          const completedState = await waitForRunStatus(baseUrl, session.id, approved.run.id, "completed");
          assert.equal(completedState.artifacts[0]?.kind, "lesson");
          assert.equal(completedState.artifacts[0]?.id.startsWith("artifact_review"), true);
          if (completedState.artifacts[0]?.kind !== "lesson") {
            throw new Error("expected lesson artifact");
          }
          assert.equal(completedState.artifacts[0].roadmapId, "review_task-core-1");
          assert.equal(completedState.artifacts[0].nodeId, "task-core-1");
        },
        {
          ...testConfig,
          orchestratorBaseUrl: `http://127.0.0.1:${address.port}`,
          orchestratorTimeoutMs: 1000,
        },
      );
    } finally {
      await closeServer(orchestrator);
    }
  });
});

async function createSession(baseUrl: string, title = "Test session", headers: Record<string, string> = {}) {
  const response = await fetch(`${baseUrl}/sessions`, {
    method: "POST",
    headers: { "Content-Type": "application/json", ...headers },
    body: JSON.stringify({ title }),
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

async function waitForRunStatus(baseUrl: string, sessionId: string, runId: string, status: string) {
  for (let attempt = 0; attempt < 40; attempt += 1) {
    const response = await fetch(`${baseUrl}/sessions/${sessionId}/state`);
    const state = (await response.json()) as {
      activeRun?: { id: string; status: string };
      pendingActions: Array<{ id: string }>;
      artifacts: Array<{ kind: string; id: string; roadmapId?: string; nodeId?: string }>;
    };
    if (state.activeRun?.id === runId && state.activeRun.status === status) {
      return state;
    }
    if (!state.activeRun && status === "completed") {
      return state;
    }
    await new Promise((resolve) => setTimeout(resolve, 50));
  }
  throw new Error(`Timed out waiting for run ${runId} to reach ${status}`);
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

function readRequestBody(request: IncomingMessage) {
  return new Promise<string>((resolve, reject) => {
    let body = "";
    request.on("data", (chunk: Buffer) => {
      body += chunk.toString("utf8");
    });
    request.on("end", () => resolve(body));
    request.on("error", reject);
  });
}
