# Orchestrator

Orchestrator la API dieu phoi trung tam cua he thong tu hoc. Service nay nhan muc tieu hoc tap tu client/gateway, lap ke hoach bang LLM, chon MCP tool phu hop, thuc thi tung buoc, luu ngu canh phien va tra ve ket qua co cau truc.

## Vai tro

- Ket noi cac MCP server bat buoc/mac dinh: `database`, `resource`, `roadmap`, `lesson`.
- Doc catalog tool tu tung MCP server va dung catalog do de lap ke hoach hanh dong.
- Quan ly vong doi agent: planning, binding input/output, tool execution, reasoning, human approval, re-plan va final output.
- Chen `authContext` da duoc tin cay vao tool can quyen de tranh de LLM tu tao quyen truy cap.
- Theo doi readiness cua cac MCP server de biet he thong da san sang xu ly workflow hay chua.

## HTTP API

Endpoint mac dinh:

```txt
http://<ORCHESTRATOR_HOST>:<ORCHESTRATOR_PORT>
```

Default local:

```txt
ORCHESTRATOR_HOST=0.0.0.0
ORCHESTRATOR_PORT=3001
```

API chinh:

```txt
GET  /health      Kiem tra process Orchestrator con song.
GET  /ready       Kiem tra cac MCP server bat buoc da ket noi va ready.
GET  /mcp/tools   Liet ke MCP server, tool count, tool names va trang thai readiness.
POST /agent/run   Tao agent run moi tu goal, session_id va auth_context.
POST /agent/resume Tiep tuc session dang doi human approval.
```

## Agent flow

```txt
1. Planning: LLM bien goal thanh danh sach PlanStep.
2. Binding: LLM hoac context resolver anh xa input/output cho tung step.
3. Execution: goi MCP tool, reasoning step, hoac tao human approval gate.
4. Evaluation: tiep tuc, doi user, re-plan, finish hoac fail.
5. Final output: gom cac ket qua duoc step danh dau final_output.
```

## Kieu action cua agent

```txt
ToolCall      Goi tool tren MCP server cu the.
Reasoning     Dung LLM de tong hop, bien doi hoac giai thich ket qua.
HumanApproval Tam dung workflow va cho /agent/resume voi approve/reject/revise.
```

## Bien moi truong quan trong

```txt
ORCHESTRATOR_MCP_SERVERS=database,resource,roadmap,lesson
ORCHESTRATOR_OPTIONAL_MCP_SERVERS=
SERVER_<NAME>_HOST=127.0.0.1
SERVER_<NAME>_PORT=3000
SERVER_<NAME>_NAME=database
SERVER_<NAME>_DESCRIPTION=...
AGENT_MAX_STEPS=8
AGENT_MAX_REPLANS=2
```

## Y nghia

Orchestrator la noi ghep cac nang luc rieng le thanh workflow hoc tap dau-cuoi. Roadmap MCP tao lo trinh, Resource MCP/Service cung cap bang chung tai lieu, Lesson MCP tao bai hoc, Database MCP luu du lieu; Orchestrator quyet dinh thu tu goi tool va dam bao cac service khong vuot boundary cua minh.
