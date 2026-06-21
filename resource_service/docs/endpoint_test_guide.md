# Resource Platform Endpoint Test Guide

Base URL mac dinh:

```txt
Resource Service: http://127.0.0.1:3200
Resource MCP:     http://127.0.0.1:3300/mcp
```

Moi REST response cua Resource Service dung envelope:

```json
{
  "success": true,
  "data": {},
  "error": null,
  "meta": {
    "requestId": "req_...",
    "timestamp": "..."
  }
}
```

## Run

```powershell
cd D:\Exercise\DATN\resource_service
cargo run
```

```powershell
cd D:\Exercise\DATN\mcp_server
cargo run -p resource_sv
```

## Health And Schema

| Method | Path | Y nghia |
| --- | --- | --- |
| GET | `/health` | Kiem tra process API con song. |
| GET | `/ready` | Kiem tra service san sang, gom ket noi database. |
| GET | `/metrics` | Tra ve metric co ban de theo doi service. |
| POST | `/admin/migrate` | Tao/cap nhat schema Resource Platform trong Postgres. Chay truoc khi test endpoint ghi du lieu. |

Example:

```powershell
curl http://127.0.0.1:3200/ready
curl -X POST http://127.0.0.1:3200/admin/migrate
```

## Resources

| Method | Path | Y nghia |
| --- | --- | --- |
| POST | `/resources` | Tao metadata resource thu cong, chua can content version. |
| GET | `/resources?limit=20&offset=0` | Liet ke resource co phan trang. |
| POST | `/resources/manual-ingest` | Tao resource + version + chunks tu content thu cong. Nhanh nhat de seed data test search. |
| GET | `/resources/{id}` | Lay detail resource, source, latest version va chunk count. |
| PATCH | `/resources/{id}` | Cap nhat metadata/status/difficulty/qualityScore/resource metadata. |
| GET | `/resources/{id}/versions` | Liet ke version cua resource. |
| POST | `/resources/{id}/versions` | Them version content moi va chunk lai. |
| GET | `/resources/{id}/chunks?maxChunks=10` | Lay chunks cua resource cho lesson/context. |

Manual ingest example:

```powershell
curl -X POST http://127.0.0.1:3200/resources/manual-ingest `
  -H "Content-Type: application/json" `
  -d '{
    "canonical_url": "https://example.com/react-useeffect-cleanup",
    "title": "React useEffect cleanup",
    "content": "# React useEffect cleanup\nUse cleanup functions to unsubscribe timers and event listeners.\n```js\nuseEffect(() => { return () => cleanup(); }, []);\n```",
    "summary": "Cleanup patterns for React useEffect.",
    "kind": "primary_learning",
    "format": "article",
    "language_code": "en",
    "primary_domain": "example.com",
    "is_official": false
  }'
```

## Search, Recommendation, Coverage

| Method | Path | Y nghia |
| --- | --- | --- |
| POST | `/search/chunks` | Tim chunk theo query. Alias cua resource search hien tai. |
| POST | `/search/resources` | Hybrid-style search tren chunks/resources, tra score va coverage. Co the tao gap khi low confidence. |
| POST | `/recommend/resources` | Chon resource phu hop cho topic/level/goal/requiredTypes. Dung cho Roadmap/Lesson. |
| POST | `/coverage/topic` | Danh gia topic co du nguon hay khong: good, partial, poor. |

Search example:

```powershell
curl -X POST http://127.0.0.1:3200/search/resources `
  -H "Content-Type: application/json" `
  -d '{
    "query": "React useEffect cleanup",
    "filters": {
      "language": "en",
      "difficulty": "beginner"
    },
    "limit": 10,
    "maxChunksPerResource": 2,
    "includeCoverage": true,
    "createGapOnLowConfidence": true
  }'
```

Recommend example:

```powershell
curl -X POST http://127.0.0.1:3200/recommend/resources `
  -H "Content-Type: application/json" `
  -d '{
    "topic": "React useEffect cleanup",
    "level": "beginner",
    "goal": "Build a lesson",
    "requiredTypes": ["official_reference", "primary_learning", "practice"],
    "maxResources": 5,
    "includeChunks": true
  }'
```

Coverage example:

```powershell
curl -X POST http://127.0.0.1:3200/coverage/topic `
  -H "Content-Type: application/json" `
  -d '{
    "topic": "PostgreSQL logical decoding with Debezium",
    "level": "intermediate",
    "requiredTypes": ["official_reference", "practice"]
  }'
```

## Sources And Crawl Pipeline

| Method | Path | Y nghia |
| --- | --- | --- |
| POST | `/sources` | Tao source site crawled duoc. |
| GET | `/sources?limit=20&offset=0` | Liet ke source site. |
| GET | `/sources/{id}` | Lay detail source site. |
| PATCH | `/sources/{id}` | Cap nhat source site, policy, allowed/blocked paths. |
| POST | `/crawl/seeds` | Tao seed URL cho scheduler. |
| GET | `/crawl/seeds?limit=20&offset=0` | Liet ke crawl seed. |
| POST | `/crawl/jobs` | Tao crawl job cu the. |
| GET | `/crawl/jobs/{id}` | Lay trang thai crawl job. |
| POST | `/worker/crawl/jobs/claim` | Worker claim jobs dang queued. |
| POST | `/worker/crawl/schedule` | Scheduler tao crawl run va jobs tu enabled seeds. |
| POST | `/worker/crawl/jobs/{id}/complete` | Worker danh dau job thanh cong/that bai. |
| POST | `/worker/fetch/artifacts` | Ghi fetch artifact sau khi crawl/fetch. |
| POST | `/worker/extract/process` | Extract artifact thanh resource/version/chunks. |

Source + seed example:

```powershell
curl -X POST http://127.0.0.1:3200/sources `
  -H "Content-Type: application/json" `
  -d '{
    "name": "React Docs",
    "kind": "official_docs",
    "baseUrl": "https://react.dev",
    "trustTier": 1,
    "languageHint": "en",
    "enabled": true,
    "isOfficial": true,
    "allowedPaths": ["/learn"],
    "blockedPaths": ["/blog"]
  }'
```

```powershell
curl -X POST http://127.0.0.1:3200/crawl/seeds `
  -H "Content-Type: application/json" `
  -d '{
    "seedUrl": "https://react.dev/learn/synchronizing-with-effects",
    "seedType": "url",
    "maxDepth": 1,
    "priority": 5,
    "enabled": true
  }'
```

## Extract And Embedding Pipeline

| Method | Path | Y nghia |
| --- | --- | --- |
| POST | `/worker/fetch/artifacts` | Luu raw body, content type, checksum cua fetch result. |
| POST | `/worker/extract/process` | Normalize HTML/Markdown/plain text, tao resource version va chunk. |
| POST | `/worker/enrichment/resources/{id}` | Chay enrichment rule-based cho topic/concept/role/outcomes. |
| POST | `/embedding/models` | Dang ky embedding model metadata. |
| GET | `/embedding/models` | Liet ke embedding model. |
| GET | `/worker/embedding/chunks/pending?limit=50` | Lay chunks chua co embedding theo model default/selected. |

Embedding model example:

```powershell
curl -X POST http://127.0.0.1:3200/embedding/models `
  -H "Content-Type: application/json" `
  -d '{
    "provider": "openai",
    "name": "text-embedding-3-small",
    "dimensions": 1536,
    "metric": "cosine",
    "isDefault": true
  }'
```

Note: phase hien tai chua expose endpoint insert vector embedding truc tiep de tranh truyen sai kieu param/vector.

## Gap And Research Candidate Flow

| Method | Path | Y nghia |
| --- | --- | --- |
| POST | `/gaps` | Bao thieu resource cho topic/resource types. Tu tao research task lien quan. |
| GET | `/gaps?limit=20&offset=0` | Liet ke gaps. |
| GET | `/gaps/{id}` | Lay detail gap. |
| POST | `/gaps/{id}/resolve` | Danh dau gap da xu ly. |
| POST | `/research/tasks` | Tao research task cho topic. |
| GET | `/research/tasks?limit=20&offset=0` | Liet ke research tasks. |
| GET | `/research/tasks/{id}` | Lay detail research task. |
| POST | `/research/candidates` | Them candidate URL, service tinh deterministic score. |
| GET | `/research/candidates?limit=20&offset=0` | Liet ke candidates. |
| GET | `/research/candidates/{id}` | Lay detail candidate. |
| POST | `/research/candidates/{id}/approve` | Approve candidate theo policy, tao source/seed/job neu can. |
| POST | `/research/candidates/{id}/reject` | Reject candidate voi reason. |

Report gap example:

```powershell
curl -X POST http://127.0.0.1:3200/gaps `
  -H "Content-Type: application/json" `
  -d '{
    "topic": "PostgreSQL logical decoding with Debezium",
    "level": "intermediate",
    "missingTypes": ["practice", "project"],
    "reason": "Search coverage is partial and lacks applied examples."
  }'
```

Research task example:

```powershell
curl -X POST http://127.0.0.1:3200/research/tasks `
  -H "Content-Type: application/json" `
  -d '{
    "topic": "React useEffect cleanup",
    "language": "en",
    "priority": 5,
    "targetResourceTypes": ["official_reference", "practice"]
  }'
```

Candidate example:

```powershell
curl -X POST http://127.0.0.1:3200/research/candidates `
  -H "Content-Type: application/json" `
  -d '{
    "researchTaskId": "<researchTaskId>",
    "url": "https://react.dev/learn/synchronizing-with-effects",
    "title": "Synchronizing with Effects",
    "snippet": "Official React guide covering effect cleanup.",
    "metadata": {
      "language": "en"
    }
  }'
```

## Admin Review Layer

| Method | Path | Y nghia |
| --- | --- | --- |
| GET | `/admin/dashboard` | Tong hop failed jobs, open gaps, pending candidates, resources can review/outdated. |
| GET | `/admin/crawl/jobs?limit=20&offset=0` | Liet ke crawl jobs cho admin. |
| POST | `/admin/crawl/schedule` | Admin trigger scheduler. |
| POST | `/admin/crawl/jobs/{id}/retry` | Retry crawl job failed/canceled. |
| POST | `/admin/crawl/jobs/{id}/cancel` | Cancel crawl job. |
| POST | `/admin/resources/{id}/enrich` | Admin trigger enrichment cho resource. |
| POST | `/admin/resources/{id}/mark-outdated` | Tao issue/audit event va danh dau resource outdated. |
| POST | `/admin/resources/{id}/mark-needs-review` | Tao issue/audit event va danh dau needs_review. |
| POST | `/admin/resources/{id}/boost` | Tang qualityScore co audit event. |
| POST | `/admin/resources/{id}/deboost` | Giam qualityScore co audit event. |
| GET | `/admin/gaps` | Admin list gaps. |
| GET | `/admin/gaps/{id}` | Admin get gap. |
| POST | `/admin/gaps/{id}/ignore` | Ignore gap. |
| POST | `/admin/gaps/{id}/reopen` | Reopen ignored/resolved gap. |
| GET | `/admin/research/tasks` | Admin list research tasks. |
| GET | `/admin/research/tasks/{id}` | Admin get research task. |
| GET | `/admin/research/candidates` | Admin list candidates. |
| GET | `/admin/research/candidates/{id}` | Admin get candidate. |
| POST | `/admin/research/candidates/{id}/approve` | Admin approve candidate theo policy. |
| POST | `/admin/research/candidates/{id}/reject` | Admin reject candidate. |

Admin action body:

```json
{
  "reason": "Outdated content or manual quality review.",
  "actorId": "admin-user-id"
}
```

## Resource MCP Tools

Resource MCP la wrapper mong quanh Resource Service, khong connect database truc tiep.

| Tool | Y nghia |
| --- | --- |
| `search_resources` | Tim resource/chunk an toan cho agent. Goi `/search/resources`. |
| `get_resource_detail` | Lay detail resource theo `resourceId`. Goi `/resources/{id}`. |
| `get_resource_chunks` | Lay chunks gioi han cho Lesson MCP. Goi `/resources/{id}/chunks`. |
| `recommend_resources_for_topic` | Goi recommendation cho Roadmap/Lesson. Goi `/recommend/resources`. |
| `get_topic_coverage` | Kiem tra coverage good/partial/poor. Goi `/coverage/topic`. |
| `report_resource_gap` | Bao gap, khong tao crawl tuy tien. Goi `/gaps`. |
| `request_research_for_topic` | Queue research task. Goi `/research/tasks`. |
| `get_integration_contract` | Tra contract Roadmap/Lesson: coverage behavior, ResourceRef, LessonChunkContext, telemetry. |

MCP guardrails:

```txt
Khong expose run_sql.
Khong expose crawl_any_url.
Khong expose insert_raw_resource/delete_resource/update_embedding.
Khong cho approve candidate bypass policy.
Clamp limit va normalize error ok=false.
```

## Phase Coverage Checklist

| Phase | Trang thai | Ghi chu |
| --- | --- | --- |
| 01 Stabilize Resource API | Done | Envelope, health, pagination, migration, resource CRUD. |
| 02 Job/worker base | Done | Crawl job claim/complete, scheduler primitives. |
| 03 Crawl pipeline | Done | Source, seed, schedule, fetch artifact. |
| 04 Extract/normalize/dedup | Done | Extract artifact, resource version, chunks, canonical URL. |
| 05 Chunk/embedding pipeline | Done | Chunker, embedding model registry, pending chunks. Vector insert endpoint intentionally not exposed. |
| 06 Hybrid search quality | Done | Search, score breakdown, coverage, alias/token boost. |
| 07 Topic/concept enrichment | Done | Rule-based topic/concept/resource metadata enrichment. |
| 08 Recommendation layer | Done | Topic recommendations, role/type coverage. |
| 09 Gap detection | Done | Gap report/list/resolve/ignore/reopen, low-confidence gap creation. |
| 10 Research candidate flow | Done | Research tasks, candidate scoring, approve/reject flow. |
| 11 Admin review layer | Done | Dashboard, retry/cancel, resource review/outdated/boost/deboost audit. |
| 12 Resource MCP wrapper | Done | `mcp_server/resource_sv` wrapper tools and normalized errors. |
| 13 Roadmap/Lesson contract | Done | Runtime `get_integration_contract` and shared contract structs. |

## Known Boundaries

```txt
Auth/RBAC chua duoc implement.
Real crawler/downloader worker chua nam trong service; service nhan artifact tu worker.
Real embedding provider/vector persistence chua expose endpoint insert vector de tranh sai kieu param.
MCP HTTP client hien ho tro HTTP local/plain, dung cho localhost Resource Service.
```
