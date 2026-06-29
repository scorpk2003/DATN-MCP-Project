# Roadmap MCP Server

Roadmap MCP Server tao lo trinh hoc tap dua tren goal, learner context va do phu tai nguyen. Server nay khong luu database truc tiep; no tra ve roadmap draft, resource refs, validation result va persistence payload de Orchestrator tiep tuc goi Database MCP.

## Endpoint

```txt
http://<ROADMAP_MCP_HOST>:<ROADMAP_MCP_PORT>/mcp
```

Default local:

```txt
ROADMAP_MCP_HOST=127.0.0.1
ROADMAP_MCP_PORT=3100
RESOURCE_MCP_URL=http://127.0.0.1:3300/mcp
RESOURCE_SERVICE_URL=http://127.0.0.1:3200
```

## Muc dich

- Chuan hoa learning goal thanh `GoalProfile`.
- Chon roadmap blueprint theo domain, stack, target role va level.
- Tach blueprint thanh topic plan, phase, node va prerequisite edge.
- Goi Resource Service contract client de kiem tra coverage va gan `ResourceRef`.
- Validate graph de phat hien node thieu resource, cycle, edge sai, phase/node loi.
- Tao `databaseReadyPayload` va `orchestratorPersistencePlan` cho Orchestrator.

## Tools

```txt
get_roadmap_contract
get_roadmap_integration_contract
get_roadmap_blueprints
validate_roadmap_request
generate_roadmap_preview
generate_roadmap_from_goal
plan_roadmap_from_topics
estimate_roadmap_scope
create_roadmap
validate_roadmap
validate_roadmap_draft
get_roadmap_detail
update_roadmap
refresh_roadmap_resources
get_health_check
get_readiness_check
```

## Y nghia cua nhom tool

```txt
Contract/readiness
  Cong bo schema, policy, blueprint, health va readiness de Orchestrator biet cach goi.

Generation
  generate_roadmap_preview, generate_roadmap_from_goal, create_roadmap tao roadmap draft co coverage.

Topic planning
  plan_roadmap_from_topics dung khi Orchestrator da co danh sach topic ung vien.

Validation
  validate_roadmap_request, validate_roadmap, validate_roadmap_draft kiem tra input va graph.

Database descriptors
  get_roadmap_detail, update_roadmap va create/refresh payload chi tao call descriptor; Orchestrator moi la ben goi Database MCP.

Resource refresh
  refresh_roadmap_resources cap nhat coverage va resource refs cho roadmap graph da ton tai.
```

## Boundary

```txt
Khong crawl website.
Khong ghi truc tiep vao Resource DB.
Khong ghi truc tiep vao application DB.
Khong tao noi dung lesson day du.
Khong chap nhan blueprint do LLM tu bia neu khong qua registry/fallback policy.
```

## Luong tich hop de xuat

```txt
1. Orchestrator goi validate_roadmap_request.
2. Roadmap MCP chon blueprint va tao topic plan.
3. Roadmap MCP hoi Resource Service de danh gia coverage.
4. Roadmap MCP tra roadmapPreview, warnings, databaseReadyPayload.
5. Orchestrator goi Database MCP theo persistence plan neu validation dat.
```
