# Resource Service

Resource Service la HTTP API va worker backend quan ly kho tai nguyen hoc tap. Service nay luu resource, version, chunks, source catalog, crawl jobs, search/recommendation, coverage, research gaps va admin review state cho cac MCP server su dung.

## Endpoint

```txt
http://<RESOURCE_SERVICE_API_HOST>:<RESOURCE_SERVICE_API_PORT>
```

Default local:

```txt
RESOURCE_SERVICE_API_HOST=127.0.0.1
RESOURCE_SERVICE_API_PORT=3200
RESOURCE_SERVICE_HOST=127.0.0.1
RESOURCE_SERVICE_PORT=5433
RESOURCE_DB=postgres
RESOURCE_USER=postgres
RESOURCE_PASS=1234
```

Tat ca route duoc bao boi internal auth middleware. Token duoc cau hinh theo vai tro:

```txt
RESOURCE_SERVICE_ADMIN_TOKEN
RESOURCE_SERVICE_WORKER_TOKEN
RESOURCE_SERVICE_MCP_TOKEN
```

## Muc dich

- Lam source of truth cho metadata tai nguyen, versions va chunks.
- Ho tro ingest thu cong va pipeline crawl/extract/enrich/embed.
- Cung cap search, recommendation va topic coverage cho Roadmap/Lesson flow.
- Theo doi resource gap, research task, candidate va GitHub discovery.
- Cung cap API admin de review, boost/deboost, mark outdated/needs_review va quan ly crawl jobs.

## API chinh

```txt
GET  /health
GET  /ready
GET  /metrics
POST /admin/migrate

POST /resources
GET  /resources
POST /resources/manual-ingest
GET  /resources/{id}
PATCH /resources/{id}
GET  /resources/{id}/versions
POST /resources/{id}/versions
GET  /resources/{id}/chunks

POST /search/chunks
POST /search/resources
POST /recommend/resources
POST /coverage/topic

POST /sources
GET  /sources
GET  /sources/{id}
PATCH /sources/{id}
POST /crawl/seeds
GET  /crawl/seeds
POST /crawl/jobs
GET  /crawl/jobs/{id}

GET  /gaps
POST /gaps
GET  /gaps/{id}
POST /gaps/{id}/resolve
GET  /research/tasks
POST /research/tasks
GET  /research/tasks/{id}
POST /research/tasks/{id}/discover/github
GET  /research/candidates
POST /research/candidates
POST /research/candidates/{id}/approve
POST /research/candidates/{id}/reject
```

## Worker API

```txt
POST /worker/crawl/jobs/claim
POST /worker/crawl/schedule
POST /worker/crawl/jobs/{id}/complete
POST /worker/fetch/artifacts
POST /worker/extract/process
POST /worker/enrichment/resources/{id}
GET  /worker/embedding/chunks/pending
```

Y nghia: cac route nay phuc vu background worker lay crawl job, ghi fetch artifact, extract content, chunk resource, enrich metadata va chuan bi embedding. Worker config gom `RESOURCE_WORKER_ID`, batch size, poll interval, timeout va max body size.

## Admin API

```txt
GET  /admin/dashboard
GET  /admin/crawl/jobs
POST /admin/crawl/jobs/{id}/retry
POST /admin/crawl/jobs/{id}/cancel
POST /admin/resources/{id}/enrich
POST /admin/resources/{id}/mark-outdated
POST /admin/resources/{id}/mark-needs-review
POST /admin/resources/{id}/boost
POST /admin/resources/{id}/deboost
GET  /admin/gaps
POST /admin/gaps/{id}/ignore
POST /admin/gaps/{id}/reopen
GET  /admin/research/tasks
GET  /admin/research/candidates
POST /admin/research/candidates/{id}/approve
POST /admin/research/candidates/{id}/reject
```

## Y nghia voi cac MCP server

```txt
Roadmap MCP
  Dung coverage/topic va recommend/resources de biet topic nao co tai lieu du tot.

Resource MCP
  La wrapper MCP mong quanh API nay, expose tool an toan cho Orchestrator.

Lesson MCP
  Khong goi truc tiep trong v0.1; Orchestrator lay resource chunks/candidates roi dua vao lesson_create_draft.

Orchestrator
  Quyet dinh khi nao search, recommend, report gap, request research hoac yeu cau admin/worker flow.
```

## Verification

```txt
cargo check
cargo test
```
