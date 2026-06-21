use serde_json::Value;

#[path = "../src/contracts.rs"]
mod contracts;

#[test]
fn resource_mcp_readiness_contract_exposes_roadmap_and_lesson_tools() {
    let contract = contracts::integration_contract();

    assert_tool(&contract, "roadmapTools", "recommend_resources_for_topic");
    assert_tool(&contract, "roadmapTools", "get_topic_coverage");
    assert_tool(&contract, "roadmapTools", "request_research_for_topic");
    assert_tool(&contract, "lessonTools", "search_resources");
    assert_tool(&contract, "lessonTools", "get_resource_detail");
    assert_tool(&contract, "lessonTools", "get_resource_chunks");
    assert_tool(&contract, "lessonTools", "recommend_resources_for_topic");

    assert_eq!(
        contract["contracts"]["coverage"]["status"],
        Value::String("partial".to_string())
    );
    assert!(
        contract["fallback"]["lowConfidence"]
            .as_str()
            .unwrap_or_default()
            .contains("coverage.lowConfidence")
    );
}

fn assert_tool(contract: &Value, list_name: &str, tool_name: &str) {
    let tools = contract[list_name]
        .as_array()
        .unwrap_or_else(|| panic!("{list_name} should be an array"));
    assert!(
        tools.iter().any(|tool| tool.as_str() == Some(tool_name)),
        "{list_name} should contain {tool_name}"
    );
}
