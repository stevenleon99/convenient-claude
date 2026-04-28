use std::collections::HashMap;
use std::io::{BufRead, BufReader};
use std::path::Path;

use crate::types::{
    AgentEntry, SessionTokenUsage, TodoItem, TodoStatus, ToolEntry, ToolStatus, TranscriptData,
};

/// Parse a Claude Code transcript JSONL file.
pub fn parse_transcript(transcript_path: &str) -> TranscriptData {
    if transcript_path.is_empty() {
        return TranscriptData::default();
    }

    let path = Path::new(transcript_path);
    if !path.exists() {
        return TranscriptData::default();
    }

    let file = match std::fs::File::open(path) {
        Ok(f) => f,
        Err(_) => return TranscriptData::default(),
    };

    let reader = BufReader::new(file);
    let mut result = TranscriptData::default();
    let mut tool_map: HashMap<String, ToolEntry> = HashMap::new();
    let mut agent_map: HashMap<String, AgentEntry> = HashMap::new();
    let mut latest_todos: Vec<TodoItem> = Vec::new();
    let mut task_id_to_index: HashMap<String, usize> = HashMap::new();
    let mut latest_slug: Option<String> = None;
    let mut custom_title: Option<String> = None;
    let mut session_tokens = SessionTokenUsage::default();
    let mut last_compact_boundary_at: Option<chrono::DateTime<chrono::Utc>> = None;
    let mut last_compact_post_tokens: Option<u64> = None;

    for line in reader.lines() {
        let line = match line {
            Ok(l) => l,
            Err(_) => continue,
        };

        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }

        let entry: serde_json::Value = match serde_json::from_str(trimmed) {
            Ok(v) => v,
            Err(_) => continue,
        };

        // Custom title
        if entry.get("type").and_then(|v| v.as_str()) == Some("custom-title") {
            if let Some(title) = entry.get("customTitle").and_then(|v| v.as_str()) {
                custom_title = Some(title.to_string());
            }
            continue;
        }

        // Slug (session name)
        if let Some(slug) = entry.get("slug").and_then(|v| v.as_str()) {
            latest_slug = Some(slug.to_string());
        }

        let timestamp = parse_timestamp(&entry);
        let entry_type = entry.get("type").and_then(|v| v.as_str()).unwrap_or("");

        // Session start — first entry with a timestamp
        if result.session_start.is_none() && timestamp.is_some() {
            result.session_start = timestamp;
        }

        // Last assistant response time
        if entry_type == "assistant" && timestamp.is_some() {
            result.last_assistant_response_at = timestamp;
        }

        // Accumulate token usage from assistant messages
        if entry_type == "assistant" {
            if let Some(usage) = entry.pointer("/message/usage") {
                session_tokens.input_tokens += normalize_token(usage.get("input_tokens"));
                session_tokens.output_tokens += normalize_token(usage.get("output_tokens"));
                session_tokens.cache_creation_tokens +=
                    normalize_token(usage.get("cache_creation_input_tokens"));
                session_tokens.cache_read_tokens +=
                    normalize_token(usage.get("cache_read_input_tokens"));
            }
        }

        // Compact boundary tracking
        if entry_type == "system"
            && entry.get("subtype").and_then(|v| v.as_str()) == Some("compact_boundary")
        {
            if let Some(ts) = timestamp {
                if last_compact_boundary_at.is_none()
                    || ts > last_compact_boundary_at.unwrap()
                {
                    last_compact_boundary_at = Some(ts);
                    if let Some(post) = entry
                        .pointer("/compactMetadata/postTokens")
                        .and_then(|v| v.as_f64())
                    {
                        last_compact_post_tokens = Some(post as u64);
                    }
                }
            }
        }

        // Process content blocks (tool_use, tool_result)
        let content = match entry.pointer("/message/content") {
            Some(serde_json::Value::Array(arr)) => arr,
            _ => continue,
        };

        for block in content {
            let block_type = block.get("type").and_then(|v| v.as_str()).unwrap_or("");

            if block_type == "tool_use" {
                let id = match block.get("id").and_then(|v| v.as_str()) {
                    Some(id) => id.to_string(),
                    None => continue,
                };
                let name = match block.get("name").and_then(|v| v.as_str()) {
                    Some(n) => n.to_string(),
                    None => continue,
                };
                let input = block.get("input");
                let ts = timestamp.unwrap_or_else(chrono::Utc::now);

                if name == "Task" || name == "Agent" {
                    let agent_type = input
                        .and_then(|i| i.get("subagent_type"))
                        .and_then(|v| v.as_str())
                        .unwrap_or("agent")
                        .to_string();
                    let model = input
                        .and_then(|i| i.get("model"))
                        .and_then(|v| v.as_str())
                        .map(|s| s.to_string());
                    let desc = input
                        .and_then(|i| i.get("description"))
                        .and_then(|v| v.as_str())
                        .map(|s| s.to_string());

                    agent_map.insert(
                        id.clone(),
                        AgentEntry {
                            id,
                            agent_type,
                            model,
                            description: desc,
                            status: ToolStatus::Running,
                            start_time: ts,
                            end_time: None,
                        },
                    );
                } else if name == "TodoWrite" {
                    if let Some(serde_json::Value::Array(arr)) = input.and_then(|i| i.get("todos")) {
                        latest_todos.clear();
                        task_id_to_index.clear();
                        for item in arr {
                            let content = item
                                .get("content")
                                .and_then(|v| v.as_str())
                                .unwrap_or("")
                                .to_string();
                            let status = parse_todo_status(
                                item.get("status").and_then(|v| v.as_str()).unwrap_or(""),
                            );
                            latest_todos.push(TodoItem { content, status });
                        }
                    }
                } else if name == "TaskCreate" {
                    let subject = input
                        .and_then(|i| i.get("subject"))
                        .and_then(|v| v.as_str())
                        .unwrap_or("")
                        .to_string();
                    let desc = input
                        .and_then(|i| i.get("description"))
                        .and_then(|v| v.as_str())
                        .unwrap_or("")
                        .to_string();
                    let content = if subject.is_empty() {
                        desc
                    } else {
                        subject
                    };
                    let status = parse_todo_status(
                        input.and_then(|i| i.get("status")).and_then(|v| v.as_str()).unwrap_or(""),
                    );
                    latest_todos.push(TodoItem { content, status });

                    if let Some(task_id) = input.and_then(|i| i.get("taskId")).and_then(|v| v.as_str()) {
                        task_id_to_index
                            .insert(task_id.to_string(), latest_todos.len() - 1);
                    }
                } else if name == "TaskUpdate" {
                    let task_id = input.and_then(|i| i.get("taskId")).and_then(|v| v.as_str());
                    if let Some(tid) = task_id {
                        if let Some(&idx) = task_id_to_index.get(tid) {
                            if idx < latest_todos.len() {
                                if let Some(status_str) = input
                                    .and_then(|i| i.get("status"))
                                    .and_then(|v| v.as_str())
                                {
                                    latest_todos[idx].status = parse_todo_status(status_str);
                                }
                                let subject = input
                                    .and_then(|i| i.get("subject"))
                                    .and_then(|v| v.as_str())
                                    .unwrap_or("");
                                let desc = input
                                    .and_then(|i| i.get("description"))
                                    .and_then(|v| v.as_str())
                                    .unwrap_or("");
                                let content = if subject.is_empty() {
                                    desc
                                } else {
                                    subject
                                };
                                if !content.is_empty() {
                                    latest_todos[idx].content = content.to_string();
                                }
                            }
                        }
                    }
                } else {
                    // Regular tool
                    let target = extract_target(&name, input);
                    tool_map.insert(
                        id.clone(),
                        ToolEntry {
                            id,
                            name,
                            target,
                            status: ToolStatus::Running,
                            start_time: ts,
                            end_time: None,
                        },
                    );
                }
            }

            if block_type == "tool_result" {
                if let Some(tool_use_id) = block.get("tool_use_id").and_then(|v| v.as_str()) {
                    let is_error = block
                        .get("is_error")
                        .and_then(|v| v.as_bool())
                        .unwrap_or(false);
                    let ts = timestamp;

                    if let Some(tool) = tool_map.get_mut(tool_use_id) {
                        tool.status = if is_error {
                            ToolStatus::Error
                        } else {
                            ToolStatus::Completed
                        };
                        tool.end_time = ts;
                    }

                    if let Some(agent) = agent_map.get_mut(tool_use_id) {
                        agent.status = ToolStatus::Completed;
                        agent.end_time = ts;
                    }
                }
            }
        }
    }

    // Keep last 20 tools, last 10 agents
    let mut tools: Vec<ToolEntry> = tool_map.into_values().collect();
    let tools_len = tools.len();
    if tools_len > 20 {
        tools = tools.split_off(tools_len - 20);
    }

    let mut agents: Vec<AgentEntry> = agent_map.into_values().collect();
    let agents_len = agents.len();
    if agents_len > 10 {
        agents = agents.split_off(agents_len - 10);
    }

    result.tools = tools;
    result.agents = agents;
    result.todos = latest_todos;
    result.session_name = custom_title.or(latest_slug);
    result.session_tokens = Some(session_tokens);
    result.last_compact_boundary_at = last_compact_boundary_at;
    result.last_compact_post_tokens = last_compact_post_tokens;

    result
}

fn parse_timestamp(entry: &serde_json::Value) -> Option<chrono::DateTime<chrono::Utc>> {
    entry
        .get("timestamp")
        .and_then(|v| v.as_str())
        .and_then(|s| chrono::DateTime::parse_from_rfc3339(s).ok())
        .map(|dt| dt.to_utc())
}

fn normalize_token(val: Option<&serde_json::Value>) -> u64 {
    val.and_then(|v| v.as_f64())
        .filter(|&n| n.is_finite() && n >= 0.0)
        .map(|n| n as u64)
        .unwrap_or(0)
}

fn extract_target(
    tool_name: &str,
    input: Option<&serde_json::Value>,
) -> Option<String> {
    let input = input?;
    match tool_name {
        "Read" | "Write" | "Edit" => input
            .get("file_path")
            .or_else(|| input.get("path"))
            .and_then(|v| v.as_str())
            .map(|s| s.to_string()),
        "Glob" => input.get("pattern").and_then(|v| v.as_str()).map(|s| s.to_string()),
        "Grep" => input.get("pattern").and_then(|v| v.as_str()).map(|s| s.to_string()),
        "Bash" => input.get("command").and_then(|v| v.as_str()).map(|cmd| {
            if cmd.len() > 30 {
                format!("{}...", &cmd[..30])
            } else {
                cmd.to_string()
            }
        }),
        "Skill" => input
            .get("skill")
            .and_then(|v| v.as_str())
            .filter(|s| !s.trim().is_empty())
            .map(|s| s.to_string()),
        _ => None,
    }
}

fn parse_todo_status(s: &str) -> TodoStatus {
    match s {
        "pending" | "not_started" => TodoStatus::Pending,
        "in_progress" | "running" => TodoStatus::InProgress,
        "completed" | "complete" | "done" => TodoStatus::Completed,
        _ => TodoStatus::Pending,
    }
}
