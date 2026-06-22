# Resource MCP Server

Thin MCP wrapper around `resource_service`.

Endpoint:

```txt
http://<SERVER_RESOURCE_MCP_HOST>:<SERVER_RESOURCE_MCP_PORT>/mcp
```

Default Resource Service target:

```txt
RESOURCE_SERVICE_BASE_URL=http://127.0.0.1:3200
```

Exposed tools:

```txt
search_resources
get_resource_detail
get_resource_chunks
recommend_resources_for_topic
get_topic_coverage
report_resource_gap
request_research_for_topic
discover_github_candidates
get_integration_contract
```

Guardrails:

```txt
No direct database access.
No raw SQL tools.
No arbitrary crawl tool.
No direct delete, embedding update, candidate approval, or quality-score mutation.
GitHub discovery creates pending research candidates only; Resource Service approval decides trust.
Errors are normalized as ok=false JSON payloads.
Search/recommend limits are clamped before forwarding to Resource Service.
```

Roadmap MCP should use `recommend_resources_for_topic`, `get_topic_coverage`, and
`request_research_for_topic`.

Lesson MCP should use `get_resource_chunks`, `search_resources`,
`get_resource_detail`, and `recommend_resources_for_topic`.
