# Agent Gateway

Agent Gateway là service trung gian giữa React Frontend và Orchestrator Agent. Frontend không gọi trực tiếp MCP Server hoặc Orchestrator; mọi agent flow đi qua Gateway để được validate, quản lý session/run, stream event, và chuẩn hóa output thành dữ liệu UI có cấu trúc.

## 1. Frontend tương tác với Gateway

Frontend gọi Gateway qua HTTP và SSE.

Luồng chính:

1. Frontend tạo session bằng `POST /sessions`.
2. Frontend gửi user intent bằng `POST /sessions/:sessionId/intents`.
3. Gateway trả về `runId` và `streamUrl`.
4. Frontend mở `EventSource` tới `GET /sessions/:sessionId/runs/:runId/stream`.
5. Gateway stream typed events về Frontend.
6. Frontend render UI dựa trên event/artifact, không parse raw LLM output.

Ví dụ intent:

```json
{
  "intent": {
    "type": "goal.submitted",
    "payload": {
      "goal": "Tôi muốn học Rust MCP trong 8 tuần"
    }
  }
}
```

Các event Frontend có thể nhận:

- `run.status_changed`
- `agent.message`
- `agent.thinking`
- `tool.started`
- `tool.completed`
- `artifact.created`
- `ui.action_required`
- `error`

Frontend cũng có thể reload state bằng:

```txt
GET /sessions/:sessionId/state
```

## 2. Orchestrator tương tác với Gateway

Ở MVP hiện tại, Orchestrator không chủ động gọi Gateway. Gateway là bên gọi Orchestrator.

Khi nhận intent từ Frontend, Gateway sẽ:

1. Validate intent bằng Zod.
2. Tạo run mới.
3. Map typed intent thành prompt/goal nội bộ.
4. Gọi Orchestrator qua:

```txt
POST ${ORCHESTRATOR_BASE_URL}/agent/run
```

Payload gửi sang Orchestrator:

```json
{
  "goal": "...",
  "session_id": "session_xxx"
}
```

Orchestrator xử lý planning/tool execution/MCP calls như hiện tại và trả response cuối về Gateway.

Gateway không yêu cầu Orchestrator stream native trong MVP. Gateway tự emit lifecycle events như `tool.started`, `tool.completed`, `run.status_changed` để Frontend có trải nghiệm realtime cơ bản.

## 3. Gateway hoạt động như thế nào

Gateway gồm các phần chính:

- **Protocol schemas**: định nghĩa intent, event, artifact, action, session state bằng Zod.
- **Session store**: lưu session, run, messages, artifacts, pending actions trong memory.
- **Event bus**: publish/subscribe event theo từng run.
- **SSE stream**: gửi event envelope về Frontend qua `text/event-stream`.
- **Intent adapter**: chuyển typed intent thành goal string cho Orchestrator.
- **Orchestrator client**: gọi `POST /agent/run`, xử lý timeout và lỗi.
- **Output adapter**: chuẩn hóa output từ Orchestrator thành UI artifact nếu nhận diện được.

Luồng xử lý một intent:

```txt
Frontend
  -> POST /sessions/:sessionId/intents
  -> Gateway validate intent
  -> Gateway create run
  -> Gateway publish queued/running events
  -> Gateway call Orchestrator
  -> Orchestrator returns output
  -> Gateway normalize output
  -> Gateway stream artifact/message/error events
  -> Gateway mark run completed/failed/waiting_for_user
```

Gateway hiện dùng in-memory store, phù hợp MVP/local development. Khi production hóa, phần store/event bus có thể thay bằng PostgreSQL, Redis, hoặc message queue mà không cần đổi contract với Frontend.

## Local commands

```bash
npm install
npm run dev
```

Gateway mặc định chạy tại:

```txt
http://localhost:4000
```

Health check:

```txt
GET /health
```
