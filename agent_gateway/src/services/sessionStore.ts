import type {
  AgentEvent,
  AgentRun,
  AgentSession,
  AgentSessionState,
  AgentStreamEnvelope,
  ChatMessage,
  UIActionRequest,
  UIArtifact,
} from "../protocol/index.js";
import { makeId, nowIso } from "./id.js";

type SessionRecord = {
  session: AgentSession;
  runs: Map<string, AgentRun>;
  messages: ChatMessage[];
  artifacts: Map<string, UIArtifact>;
  pendingActions: Map<string, UIActionRequest>;
  timeline: AgentSessionState["timeline"];
  envelopes: Map<string, AgentStreamEnvelope[]>;
  sequences: Map<string, number>;
};

export class SessionStore {
  private readonly sessions = new Map<string, SessionRecord>();

  createSession(input: { userId?: string; title?: string; metadata?: Record<string, unknown> }) {
    const timestamp = nowIso();
    const session: AgentSession = {
      id: makeId("session"),
      userId: input.userId,
      title: input.title,
      status: "active",
      createdAt: timestamp,
      updatedAt: timestamp,
      metadata: input.metadata,
    };

    this.sessions.set(session.id, {
      session,
      runs: new Map(),
      messages: [],
      artifacts: new Map(),
      pendingActions: new Map(),
      timeline: [],
      envelopes: new Map(),
      sequences: new Map(),
    });

    return session;
  }

  listSessions(input: { userId?: string; limit?: number }) {
    const limit = input.limit ?? 50;
    return [...this.sessions.values()]
      .map((record) => record.session)
      .filter((session) => !input.userId || session.userId === input.userId)
      .sort((a, b) => b.updatedAt.localeCompare(a.updatedAt))
      .slice(0, limit);
  }

  getRecord(sessionId: string) {
    return this.sessions.get(sessionId);
  }

  getSession(sessionId: string) {
    return this.sessions.get(sessionId)?.session;
  }

  createRun(sessionId: string, parentRunId?: string) {
    const record = this.sessions.get(sessionId);
    if (!record) {
      return null;
    }

    const timestamp = nowIso();
    const run: AgentRun = {
      id: makeId("run"),
      sessionId,
      parentRunId,
      status: "queued",
      startedAt: timestamp,
    };

    record.runs.set(run.id, run);
    record.envelopes.set(run.id, []);
    record.sequences.set(run.id, 0);
    record.session.updatedAt = timestamp;
    return run;
  }

  getRun(sessionId: string, runId: string) {
    return this.sessions.get(sessionId)?.runs.get(runId);
  }

  updateRun(sessionId: string, runId: string, patch: Partial<AgentRun>) {
    const run = this.getRun(sessionId, runId);
    const record = this.sessions.get(sessionId);
    if (!run || !record) {
      return null;
    }

    Object.assign(run, patch);
    record.session.updatedAt = nowIso();
    return run;
  }

  appendEnvelope(sessionId: string, runId: string, event: AgentEvent) {
    const record = this.sessions.get(sessionId);
    if (!record) {
      return null;
    }

    const nextSequence = (record.sequences.get(runId) ?? 0) + 1;
    const envelope: AgentStreamEnvelope = {
      id: makeId("evt"),
      sessionId,
      runId,
      sequence: nextSequence,
      timestamp: nowIso(),
      event,
    };

    record.sequences.set(runId, nextSequence);
    record.envelopes.set(runId, [...(record.envelopes.get(runId) ?? []), envelope]);
    this.applyEvent(record, envelope);
    record.session.updatedAt = envelope.timestamp;
    return envelope;
  }

  getRunEnvelopes(sessionId: string, runId: string) {
    return this.sessions.get(sessionId)?.envelopes.get(runId) ?? [];
  }

  getState(sessionId: string): AgentSessionState | null {
    const record = this.sessions.get(sessionId);
    if (!record) {
      return null;
    }

    const activeRun = [...record.runs.values()]
      .filter((run) => ["queued", "running", "waiting_for_user"].includes(run.status))
      .sort((a, b) => b.startedAt.localeCompare(a.startedAt))[0];

    return {
      session: record.session,
      activeRun,
      messages: record.messages,
      artifacts: [...record.artifacts.values()],
      pendingActions: [...record.pendingActions.values()].filter((action) => action.status === "pending"),
      timeline: record.timeline,
    };
  }

  addUserMessage(sessionId: string, runId: string, content: string) {
    const record = this.sessions.get(sessionId);
    if (!record) {
      return;
    }

    record.messages.push({
      id: makeId("msg"),
      role: "user",
      content,
      runId,
      createdAt: nowIso(),
    });
  }

  resolveAction(sessionId: string, actionId: string) {
    const record = this.sessions.get(sessionId);
    const action = record?.pendingActions.get(actionId);
    if (!record || !action) {
      return null;
    }

    const resolved = {
      ...action,
      status: "resolved" as const,
      resolvedAt: nowIso(),
    };
    record.pendingActions.set(actionId, resolved);
    return resolved;
  }

  getAction(sessionId: string, actionId: string) {
    return this.sessions.get(sessionId)?.pendingActions.get(actionId) ?? null;
  }

  private applyEvent(record: SessionRecord, envelope: AgentStreamEnvelope) {
    const { event } = envelope;

    if (event.type === "agent.message") {
      record.messages.push({
        id: makeId("msg"),
        role: "agent",
        content: event.message,
        runId: envelope.runId,
        createdAt: envelope.timestamp,
      });
    }

    if (event.type === "agent.thinking") {
      record.timeline.push({
        id: makeId("timeline"),
        runId: envelope.runId,
        type: event.type,
        label: event.label,
        status: "started",
        createdAt: envelope.timestamp,
      });
    }

    if (event.type === "tool.started" || event.type === "tool.completed") {
      record.timeline.push({
        id: makeId("timeline"),
        runId: envelope.runId,
        type: event.type,
        label: event.displayName,
        status: event.type === "tool.completed" ? "completed" : "started",
        createdAt: envelope.timestamp,
      });
    }

    if (event.type === "artifact.created") {
      record.artifacts.set(event.artifact.id, event.artifact);
    }

    if (event.type === "artifact.updated") {
      const current = record.artifacts.get(event.artifactId);
      if (current) {
        record.artifacts.set(event.artifactId, { ...current, ...event.patch } as UIArtifact);
      }
    }

    if (event.type === "ui.action_required") {
      record.pendingActions.set(event.action.id, event.action);
    }
  }
}

export const sessionStore = new SessionStore();
