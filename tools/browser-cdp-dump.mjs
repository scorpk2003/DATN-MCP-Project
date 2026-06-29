const tabs = await fetch("http://127.0.0.1:9222/json").then((response) => response.json());
const page = tabs.find((tab) => tab.type === "page") ?? tabs[0];
const socket = new WebSocket(page.webSocketDebuggerUrl);
await new Promise((resolve, reject) => {
  socket.addEventListener("open", resolve, { once: true });
  socket.addEventListener("error", reject, { once: true });
});
let id = 0;
const callbacks = new Map();
socket.addEventListener("message", (message) => {
  const payload = JSON.parse(message.data);
  if (payload.id && callbacks.has(payload.id)) {
    callbacks.get(payload.id)(payload);
    callbacks.delete(payload.id);
  }
});
const send = (method, params = {}) =>
  new Promise((resolve, reject) => {
    const callId = ++id;
    callbacks.set(callId, (payload) => {
      if (payload.error) {
        reject(new Error(payload.error.message));
        return;
      }
      resolve(payload.result);
    });
    socket.send(JSON.stringify({ id: callId, method, params }));
  });
await send("Runtime.enable");
const result = await send("Runtime.evaluate", {
  expression: "({ href: location.href, title: document.title, text: document.body.innerText, html: document.body.innerHTML.slice(0, 1000) })",
  returnByValue: true,
});
socket.close();
console.log(JSON.stringify(result.result.value, null, 2));
