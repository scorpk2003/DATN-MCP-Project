use std::{
    collections::BTreeMap,
    sync::{
        Mutex,
        atomic::{AtomicU64, Ordering},
    },
};

use serde_json::{Value, json};
use tracing::{info, warn};

#[derive(Debug, Default)]
pub struct LessonTelemetry {
    total_tool_calls: AtomicU64,
    total_tool_successes: AtomicU64,
    total_tool_errors: AtomicU64,
    tool_calls: Mutex<BTreeMap<String, u64>>,
    tool_errors: Mutex<BTreeMap<String, u64>>,
    error_codes: Mutex<BTreeMap<String, u64>>,
}

impl LessonTelemetry {
    pub fn record_tool_call(&self, tool: &str, request_id: Option<&str>) {
        self.total_tool_calls.fetch_add(1, Ordering::Relaxed);
        increment(&self.tool_calls, tool);
        info!(
            service = "lesson_mcp",
            tool = tool,
            request_id = request_id.unwrap_or(""),
            event = "tool_call_started"
        );
    }

    pub fn record_tool_success(&self, tool: &str) {
        self.total_tool_successes.fetch_add(1, Ordering::Relaxed);
        info!(
            service = "lesson_mcp",
            tool = tool,
            event = "tool_call_succeeded"
        );
    }

    pub fn record_tool_error(&self, tool: &str, code: &str, retryable: bool) {
        self.total_tool_errors.fetch_add(1, Ordering::Relaxed);
        increment(&self.tool_errors, tool);
        increment(&self.error_codes, code);
        warn!(
            service = "lesson_mcp",
            tool = tool,
            error_code = code,
            retryable = retryable,
            event = "tool_call_failed"
        );
    }

    pub fn snapshot(&self) -> Value {
        json!({
            "totalToolCalls": self.total_tool_calls.load(Ordering::Relaxed),
            "totalToolSuccesses": self.total_tool_successes.load(Ordering::Relaxed),
            "totalToolErrors": self.total_tool_errors.load(Ordering::Relaxed),
            "toolCalls": snapshot_map(&self.tool_calls),
            "toolErrors": snapshot_map(&self.tool_errors),
            "errorCodes": snapshot_map(&self.error_codes),
        })
    }
}

fn increment(map: &Mutex<BTreeMap<String, u64>>, key: &str) {
    let mut guard = map.lock().expect("telemetry mutex should not be poisoned");
    *guard.entry(key.to_string()).or_insert(0) += 1;
}

fn snapshot_map(map: &Mutex<BTreeMap<String, u64>>) -> BTreeMap<String, u64> {
    map.lock()
        .expect("telemetry mutex should not be poisoned")
        .clone()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn records_calls_successes_and_errors() {
        let telemetry = LessonTelemetry::default();

        telemetry.record_tool_call("lesson_analyze_node", Some("req-1"));
        telemetry.record_tool_success("lesson_analyze_node");
        telemetry.record_tool_call("lesson_create_draft", Some("req-2"));
        telemetry.record_tool_error("lesson_create_draft", "INSUFFICIENT_RESOURCES", false);

        let snapshot = telemetry.snapshot();
        assert_eq!(snapshot["totalToolCalls"], 2);
        assert_eq!(snapshot["totalToolSuccesses"], 1);
        assert_eq!(snapshot["totalToolErrors"], 1);
        assert_eq!(snapshot["toolCalls"]["lesson_analyze_node"], 1);
        assert_eq!(snapshot["errorCodes"]["INSUFFICIENT_RESOURCES"], 1);
    }
}
