import { createSession, sendIntent } from "./agentGatewayClient.js";

export async function startLearningFlow({ user, title, intent, metadata = {} }) {
  const sessionTitle = title || "Learning session";
  const { session } = await createSession({
    userId: user?.uid,
    title: sessionTitle,
    metadata: {
      source: "web",
      flow: intent.type,
      locale: navigator.language || "vi-VN",
      ...metadata,
    },
  });

  const accepted = await sendIntent(session.id, {
    intent,
  });

  return { session, run: accepted.run };
}
