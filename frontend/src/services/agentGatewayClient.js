import { getAuthToken, getAuthUserId } from "../auth/authService.js";
import { AGENT_GATEWAY_URL } from "../config/env.js";

function buildGatewayUrl(path) {
  if (/^https?:\/\//i.test(path)) {
    return path;
  }

  return `${AGENT_GATEWAY_URL.replace(/\/$/, "")}${path}`;
}

async function gatewayRequest(path, options = {}) {
  const token = await getAuthToken();
  const userId = getAuthUserId();
  const response = await fetch(buildGatewayUrl(path), {
    ...options,
    headers: {
      "Content-Type": "application/json",
      ...(token ? { Authorization: `Bearer ${token}` } : {}),
      ...(userId ? { "X-User-ID": userId } : {}),
      ...options.headers,
    },
  });

  if (!response.ok) {
    let message = `Gateway request failed with status ${response.status}`;
    try {
      const body = await response.json();
      message = body?.error?.message || message;
    } catch {
      // Keep the generic status message when the response is not JSON.
    }
    throw new Error(message);
  }

  if (response.status === 204) {
    return null;
  }

  return response.json();
}

export function createSession(input = {}) {
  return gatewayRequest("/sessions", {
    method: "POST",
    body: JSON.stringify(input),
  });
}

export function sendIntent(sessionId, input) {
  return gatewayRequest(`/sessions/${sessionId}/intents`, {
    method: "POST",
    body: JSON.stringify(input),
  });
}

export async function subscribeRun(sessionId, runId, onEvent, onError) {
  const token = await getAuthToken();
  const userId = getAuthUserId();
  const query = new URLSearchParams();
  if (userId) {
    query.set("userId", userId);
  }
  if (token) {
    query.set("access_token", token);
  }
  const suffix = query.toString() ? `?${query.toString()}` : "";
  const eventSource = new EventSource(buildGatewayUrl(`/sessions/${sessionId}/runs/${runId}/stream${suffix}`));

  eventSource.addEventListener("agent_event", (raw) => {
    onEvent(JSON.parse(raw.data));
  });

  eventSource.onerror = (error) => {
    onError?.(error);
    eventSource.close();
  };

  return () => eventSource.close();
}

export function getSessionState(sessionId) {
  return gatewayRequest(`/sessions/${sessionId}/state`);
}

export function respondToAction(sessionId, actionId, input) {
  return gatewayRequest(`/sessions/${sessionId}/actions/${actionId}/respond`, {
    method: "POST",
    body: JSON.stringify(input),
  });
}

export function cancelRun(sessionId, runId) {
  return gatewayRequest(`/sessions/${sessionId}/runs/${runId}/cancel`, {
    method: "POST",
  });
}
