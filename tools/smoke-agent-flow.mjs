const base = "http://127.0.0.1:4000";

const post = async (path, body) => {
  const response = await fetch(base + path, {
    method: "POST",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify(body),
  });
  if (!response.ok) {
    throw new Error(`${path} ${response.status}: ${await response.text()}`);
  }
  return response.json();
};

const get = async (path) => {
  const response = await fetch(base + path);
  if (!response.ok) {
    throw new Error(`${path} ${response.status}: ${await response.text()}`);
  }
  return response.json();
};

const waitFor = async (read) => {
  for (let attempt = 0; attempt < 50; attempt += 1) {
    const value = await read();
    if (value) {
      return value;
    }
    await new Promise((resolve) => setTimeout(resolve, 100));
  }
  throw new Error("Timed out waiting for flow state.");
};

const { session } = await post("/sessions", {
  title: "Learner CCNA e2e",
  userId: "learner-e2e",
  metadata: { source: "web", locale: "vi-VN" },
});

const run = await post(`/sessions/${session.id}/intents`, {
  intent: {
    type: "goal.submitted",
    payload: { goal: "Learn CCNA in 8 weeks" },
  },
});

const waiting = await waitFor(async () => {
  const state = await get(`/sessions/${session.id}/state`);
  return state.pendingActions.length ? state : null;
});
const action = waiting.pendingActions[0];

const resume = await post(`/sessions/${session.id}/actions/${action.id}/respond`, {
  selectedOptionId: "approve",
});

const completed = await waitFor(async () => {
  const state = await get(`/sessions/${session.id}/state`);
  return state.artifacts.length ? state : null;
});

console.log(
  JSON.stringify(
    {
      sessionId: session.id,
      firstRun: run.run.status,
      action: {
        title: action.title,
        stepId: action.metadata?.stepId,
        options: action.options.map((option) => option.id),
      },
      resumeRun: resume.run.status,
      artifacts: completed.artifacts.map((artifact) => ({
        kind: artifact.kind,
        id: artifact.id,
        title: artifact.title || artifact.topicName,
        nodes: artifact.nodes?.length,
        coverageStatus: artifact.coverageStatus,
      })),
      pendingActions: completed.pendingActions.map((item) => item.title),
      messages: completed.messages.map((message) => message.content).slice(-4),
    },
    null,
    2,
  ),
);
