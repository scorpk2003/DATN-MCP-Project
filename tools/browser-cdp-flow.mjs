const tabs = await fetch("http://127.0.0.1:9222/json").then((response) => response.json());
const page = tabs.find((tab) => tab.type === "page") ?? tabs[0];
if (!page?.webSocketDebuggerUrl) {
  throw new Error("No Chrome page target found.");
}

const socket = new WebSocket(page.webSocketDebuggerUrl);
await new Promise((resolve, reject) => {
  socket.addEventListener("open", resolve, { once: true });
  socket.addEventListener("error", reject, { once: true });
});

let id = 0;
const callbacks = new Map();
const events = [];

socket.addEventListener("message", (message) => {
  const payload = JSON.parse(message.data);
  if (payload.id && callbacks.has(payload.id)) {
    callbacks.get(payload.id)(payload);
    callbacks.delete(payload.id);
    return;
  }
  if (payload.method) {
    events.push(payload);
  }
});

const send = (method, params = {}) =>
  new Promise((resolve, reject) => {
    const callId = ++id;
    callbacks.set(callId, (payload) => {
      if (payload.error) {
        reject(new Error(`${method}: ${payload.error.message}`));
        return;
      }
      resolve(payload.result);
    });
    socket.send(JSON.stringify({ id: callId, method, params }));
  });

await send("Runtime.enable");
await send("Page.enable");
await send("Page.bringToFront");
await send("Page.navigate", { url: "http://127.0.0.1:5174/" });

const evaluate = async (expression) => {
  const result = await send("Runtime.evaluate", {
    expression,
    awaitPromise: true,
    returnByValue: true,
  });
  if (result.exceptionDetails) {
    throw new Error(result.exceptionDetails.text);
  }
  return result.result.value;
};

const waitFor = async (predicate, label) => {
  for (let attempt = 0; attempt < 80; attempt += 1) {
    const value = await predicate();
    if (value) {
      return value;
    }
    await new Promise((resolve) => setTimeout(resolve, 250));
  }
  throw new Error(`Timed out waiting for ${label}.`);
};

await waitFor(
  () => evaluate("document.body.innerText.includes('Agent workspace')"),
  "home workspace",
);

await evaluate(`(() => {
  const textarea = document.querySelector('textarea');
  const setter = Object.getOwnPropertyDescriptor(HTMLTextAreaElement.prototype, 'value').set;
  setter.call(textarea, 'Learn CCNA in 8 weeks');
  textarea.dispatchEvent(new Event('input', { bubbles: true }));
  return textarea.value;
})()`);

await evaluate(`(() => {
  const button = [...document.querySelectorAll('button')].find((item) => item.innerText.includes('Run agent'));
  button.click();
  return true;
})()`);

await waitFor(
  () => evaluate("document.body.innerText.includes('Action required') && document.body.innerText.includes('Approve')"),
  "approval action",
);

const approvalText = await evaluate("document.body.innerText");

await evaluate(`(() => {
  const button = [...document.querySelectorAll('button')].find((item) => item.innerText.trim().includes('Approve'));
  button.click();
  return true;
})()`);

await waitFor(
  () =>
    evaluate(
      "document.body.innerText.includes('CCNA standard learner roadmap') && document.body.innerText.includes('IPv4 subnetting')",
    ),
  "roadmap artifact",
);

await evaluate(`(() => {
  const button = [...document.querySelectorAll('button')].find((item) => {
    const cardText = item.parentElement?.parentElement?.innerText || '';
    return item.innerText.includes('Open lesson') && cardText.includes('IPv4 subnetting');
  });
  button.click();
  return true;
})()`);

await waitFor(
  () =>
    evaluate(
      "document.body.innerText.includes('IPv4 subnetting practice') && document.body.innerText.includes('Calculate subnet ranges')",
    ),
  "lesson artifact",
);

const finalText = await evaluate("document.body.innerText");
const result = {
  initialUrl: page.url,
  reachedWorkspace: finalText.includes("Agent workspace"),
  approvalVisibleBeforeApprove:
    approvalText.includes("Orchestrator needs approval") &&
    approvalText.includes("review_ccna_roadmap_draft"),
  roadmapVisibleAfterApprove:
    finalText.includes("CCNA standard learner roadmap") &&
    finalText.includes("Networking foundations") &&
    finalText.includes("IPv4 subnetting"),
  staleWaitingRunVisible: finalText.includes("waiting_for_user"),
  resourceReadinessVisible: finalText.includes("resource_readiness"),
  lessonVisibleAfterNodeOpen:
    finalText.includes("IPv4 subnetting practice") &&
    finalText.includes("Calculate subnet ranges") &&
    finalText.includes("Find the network address"),
  messagesVisible:
    finalText.includes("Learn CCNA in 8 weeks") &&
    finalText.includes("2 artifacts generated.") &&
    finalText.includes("lesson artifact generated."),
  bogusApprovalMessageVisible: finalText.includes("did not match a supported UI artifact"),
  visibleStatusSnippets: finalText
    .split("\\n")
    .filter((line) =>
      ["Agent workspace", "Action required", "CCNA standard learner roadmap", "resource_readiness", "2 artifacts generated."].some(
        (needle) => line.includes(needle),
      ),
    )
    .slice(0, 12),
};

socket.close();
console.log(JSON.stringify(result, null, 2));
