import { EventEmitter } from "node:events";
import type { AgentEvent, AgentStreamEnvelope } from "../protocol/index.js";
import { sessionStore } from "./sessionStore.js";

export class EventBus {
  private readonly emitter = new EventEmitter();

  publish(sessionId: string, runId: string, event: AgentEvent) {
    const envelope = sessionStore.appendEnvelope(sessionId, runId, event);
    if (envelope) {
      this.emitter.emit(this.key(sessionId, runId), envelope);
    }
    return envelope;
  }

  subscribe(sessionId: string, runId: string, listener: (envelope: AgentStreamEnvelope) => void) {
    const key = this.key(sessionId, runId);
    this.emitter.on(key, listener);
    return () => this.emitter.off(key, listener);
  }

  private key(sessionId: string, runId: string) {
    return `${sessionId}:${runId}`;
  }
}

export const eventBus = new EventBus();
