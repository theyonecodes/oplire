use serde_json::{json, Value};

fn anthropic_to_opencode_request(body: &Value) -> Value {
    let model = body.get("model").and_then(|v| v.as_str()).unwrap_or("");
    let max_tokens = body.get("max_tokens").and_then(|v| v.as_u64()).unwrap_or(4096);
    let stream = body.get("stream").and_then(|v| v.as_bool()).unwrap_or(false);
    let temperature = body.get("temperature").cloned();
    let top_p = body.get("top_p").cloned();

    let mut messages: Vec<Value> = Vec::new();

    if let Some(system) = body.get("system") {
        let system_content = match system {
            Value::String(s) => s.clone(),
            Value::Array(arr) => arr
                .iter()
                .filter_map(|c| c.get("text").and_then(|t| t.as_str()))
                .collect::<Vec<_>>()
                .join("\n"),
            _ => String::new(),
        };
        if !system_content.is_empty() {
            messages.push(json!({
                "role": "system",
                "content": system_content
            }));
        }
    }

    if let Some(msgs) = body.get("messages").and_then(|v| v.as_array()) {
        for msg in msgs {
            let role = msg.get("role").and_then(|v| v.as_str()).unwrap_or("user");
            let content = match msg.get("content") {
                Some(Value::String(s)) => s.clone(),
                Some(Value::Array(arr)) => arr
                    .iter()
                    .filter_map(|c| c.get("text").and_then(|t| t.as_str()))
                    .collect::<Vec<_>>()
                    .join("\n"),
                Some(Value::Null) | None => String::new(),
                _ => String::new(),
            };
            let openai_role = match role {
                "assistant" => "assistant",
                _ => "user",
            };
            messages.push(json!({
                "role": openai_role,
                "content": content
            }));
        }
    }

    let mut req = json!({
        "model": model,
        "messages": messages,
        "stream": stream,
        "max_tokens": max_tokens
    });

    if let Some(obj) = req.as_object_mut() {
        if let Some(t) = temperature {
            obj.insert("temperature".to_string(), t);
        }
        if let Some(p) = top_p {
            obj.insert("top_p".to_string(), p);
        }
    }

    req
}

fn opencode_response_to_anthropic(body: &Value, model: &str) -> Value {
    let mut text = String::new();
    let mut finish_reason = "stop";

    if let Some(choices) = body.get("choices").and_then(|v| v.as_array()) {
        if let Some(choice) = choices.first() {
            if let Some(message) = choice.get("message") {
                if let Some(content) = message.get("content").and_then(|v| v.as_str()) {
                    text = content.to_string();
                }
            }
            if let Some(reason) = choice.get("finish_reason").and_then(|v| v.as_str()) {
                finish_reason = match reason {
                    "stop" => "end_turn",
                    "length" => "max_tokens",
                    _ => "end_turn",
                };
            }
        }
    }

    let usage = body.get("usage").cloned().unwrap_or(json!({
        "input_tokens": 0,
        "output_tokens": 0
    }));

    json!({
        "id": "msg_test",
        "type": "message",
        "role": "assistant",
        "content": [{
            "type": "text",
            "text": text
        }],
        "model": model,
        "stop_reason": finish_reason,
        "stop_sequence": null,
        "usage": usage
    })
}

fn opencode_stream_to_anthropic(line: &str, _model: &str) -> Option<String> {
    if line.is_empty() || line.starts_with(':') {
        return None;
    }

    if let Some(data) = line.strip_prefix("data: ") {
        if data == "[DONE]" {
            return Some(
                "event: message_stop\ndata: {\"type\":\"message_stop\"}\n\n".to_string(),
            );
        }

        if let Ok(val) = serde_json::from_str::<Value>(data) {
            if let Some(choices) = val.get("choices").and_then(|v| v.as_array()) {
                if let Some(choice) = choices.first() {
                    let delta_text = choice
                        .get("delta")
                        .and_then(|d| d.get("content"))
                        .and_then(|c| c.as_str())
                        .unwrap_or("");

                    let finish_reason = choice
                        .get("finish_reason")
                        .and_then(|v| v.as_str());

                    if !delta_text.is_empty() {
                        return Some(format!(
                            "event: content_block_delta\ndata: {{\"type\":\"content_block_delta\",\"index\":0,\"delta\":{{\"type\":\"text_delta\",\"text\":\"{}\"}}}}\n\n",
                            delta_text
                        ));
                    }

                    if let Some(reason) = finish_reason {
                        let stop_reason = match reason {
                            "stop" => "end_turn",
                            "length" => "max_tokens",
                            _ => "end_turn",
                        };
                        return Some(format!(
                            "event: message_delta\ndata: {{\"type\":\"message_delta\",\"delta\":{{\"stop_reason\":\"{}\",\"stop_sequence\":null}}}}\n\n",
                            stop_reason
                        ));
                    }
                }
            }
        }
    }

    None
}

fn escape_json_string(s: &str) -> String {
    s.replace('\\', "\\\\")
        .replace('"', "\\\"")
        .replace('\n', "\\n")
        .replace('\r', "\\r")
        .replace('\t', "\\t")
}

#[test]
fn test_anthropic_to_opencode_basic() {
    let input = json!({
        "model": "glm-4.7-free",
        "messages": [
            {"role": "user", "content": "Hello"}
        ],
        "max_tokens": 1024
    });
    
    let result = anthropic_to_opencode_request(&input);
    assert_eq!(result.get("model").and_then(|v| v.as_str()), Some("glm-4.7-free"));
    assert_eq!(result.get("max_tokens").and_then(|v| v.as_u64()), Some(1024));
    let messages = result.get("messages").and_then(|v| v.as_array()).unwrap();
    assert_eq!(messages.len(), 1);
    assert_eq!(messages[0].get("role").and_then(|v| v.as_str()), Some("user"));
    assert_eq!(messages[0].get("content").and_then(|v| v.as_str()), Some("Hello"));
}

#[test]
fn test_anthropic_to_opencode_with_system() {
    let input = json!({
        "model": "glm-4.7-free",
        "system": "You are a helpful assistant.",
        "messages": [
            {"role": "user", "content": "Hi"}
        ]
    });
    
    let result = anthropic_to_opencode_request(&input);
    let messages = result.get("messages").and_then(|v| v.as_array()).unwrap();
    assert_eq!(messages.len(), 2);
    assert_eq!(messages[0].get("role").and_then(|v| v.as_str()), Some("system"));
    assert_eq!(messages[0].get("content").and_then(|v| v.as_str()), Some("You are a helpful assistant."));
}

#[test]
fn test_anthropic_to_opencode_with_temperature() {
    let input = json!({
        "model": "glm-4.7-free",
        "messages": [{"role": "user", "content": "Test"}],
        "temperature": 0.7,
        "top_p": 0.9
    });
    
    let result = anthropic_to_opencode_request(&input);
    assert!(result.get("temperature").is_some());
    assert!(result.get("top_p").is_some());
}

#[test]
fn test_opencode_response_to_anthropic() {
    let input = json!({
        "choices": [{
            "message": {"content": "Hello!"},
            "finish_reason": "stop"
        }],
        "usage": {
            "prompt_tokens": 10,
            "completion_tokens": 5
        }
    });
    
    let result = opencode_response_to_anthropic(&input, "glm-4.7-free");
    assert_eq!(result.get("type").and_then(|v| v.as_str()), Some("message"));
    assert_eq!(result.get("role").and_then(|v| v.as_str()), Some("assistant"));
    assert_eq!(result.get("model").and_then(|v| v.as_str()), Some("glm-4.7-free"));
    assert_eq!(result.get("stop_reason").and_then(|v| v.as_str()), Some("end_turn"));
    
    let content = result.get("content").and_then(|v| v.as_array()).unwrap();
    assert_eq!(content[0].get("type").and_then(|v| v.as_str()), Some("text"));
    assert_eq!(content[0].get("text").and_then(|v| v.as_str()), Some("Hello!"));
}

#[test]
fn test_opencode_stream_to_anthropic_done() {
    let result = opencode_stream_to_anthropic("data: [DONE]", "glm-4.7-free");
    assert!(result.is_some());
    assert!(result.unwrap().contains("message_stop"));
}

#[test]
fn test_opencode_stream_to_anthropic_content() {
    let input = "data: {\"choices\": [{\"delta\": {\"content\": \"Hello\"}, \"finish_reason\": null}]}";
    let result = opencode_stream_to_anthropic(input, "glm-4.7-free");
    assert!(result.is_some());
    let output = result.unwrap();
    assert!(output.contains("content_block_delta"));
    assert!(output.contains("Hello"));
}

#[test]
fn test_opencode_stream_to_anthropic_finish() {
    let input = "data: {\"choices\": [{\"delta\": {}, \"finish_reason\": \"stop\"}]}";
    let result = opencode_stream_to_anthropic(input, "glm-4.7-free");
    assert!(result.is_some());
    let output = result.unwrap();
    assert!(output.contains("message_delta"));
    assert!(output.contains("end_turn"));
}

#[test]
fn test_opencode_stream_to_anthropic_empty() {
    assert!(opencode_stream_to_anthropic("", "glm-4.7-free").is_none());
    assert!(opencode_stream_to_anthropic(": comment", "glm-4.7-free").is_none());
}

#[test]
fn test_escape_json_string() {
    assert_eq!(escape_json_string("hello"), "hello");
    assert_eq!(escape_json_string("he\"llo"), "he\\\"llo");
    assert_eq!(escape_json_string("line\nbreak"), "line\\nbreak");
    assert_eq!(escape_json_string("tab\there"), "tab\\there");
    assert_eq!(escape_json_string("back\\slash"), "back\\\\slash");
}