use serde_json::Value;

#[test]
fn resource_eval_dataset_v0_2_has_required_shape() {
    let raw = include_str!("../evaluation/resource_eval_dataset_v0_2.json");
    let dataset: Value =
        serde_json::from_str(raw).expect("evaluation dataset should be valid JSON");
    assert_eq!(dataset["version"], "resource-platform-v0.2");

    let topics = dataset["topics"]
        .as_array()
        .expect("topics should be an array");
    assert!(topics.len() >= 30);

    for topic in topics {
        assert_non_empty_string(topic, "topic");
        assert_one_of(
            topic,
            "group",
            &["popular", "intermediate", "niche", "project"],
        );
        assert_one_of(topic, "expectedCoverage", &["good", "partial", "poor"]);
        assert!(
            topic["requiredTypes"]
                .as_array()
                .map(|items| !items.is_empty())
                .unwrap_or(false),
            "requiredTypes should be a non-empty array for {topic:?}"
        );
        assert!(
            topic["expectedOfficialDomains"].as_array().is_some(),
            "expectedOfficialDomains should be an array for {topic:?}"
        );
        assert!(
            topic["expectedMinResources"].as_i64().unwrap_or_default() >= 2,
            "expectedMinResources should be at least 2 for {topic:?}"
        );
        assert!(
            topic["expectedGapCreated"].as_bool().is_some(),
            "expectedGapCreated should be boolean for {topic:?}"
        );
    }
}

fn assert_non_empty_string(value: &Value, field: &str) {
    assert!(
        value[field]
            .as_str()
            .map(|item| !item.trim().is_empty())
            .unwrap_or(false),
        "{field} should be a non-empty string for {value:?}"
    );
}

fn assert_one_of(value: &Value, field: &str, allowed: &[&str]) {
    let actual = value[field]
        .as_str()
        .unwrap_or_else(|| panic!("{field} should be a string for {value:?}"));
    assert!(
        allowed.contains(&actual),
        "{field}={actual} is not allowed for {value:?}"
    );
}
