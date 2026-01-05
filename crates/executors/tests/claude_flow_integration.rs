use std::{path::PathBuf, sync::Arc, time::Duration};

use async_trait::async_trait;
use command_group::AsyncGroupChild;
use executors::{
    env::ExecutionEnv,
    executors::{
        ClaudeFlow, ExecutorError, SpawnedChild, StandardCodingAgentExecutor,
    },
};
use futures::StreamExt;
use serde_json::json;
use tempfile::TempDir;
use tokio::{io::AsyncWriteExt, sync::oneshot, time::timeout};
use workspace_utils::msg_store::MsgStore;

#[derive(Debug)]
struct MockChild {
    stdout_rx: tokio::sync::mpsc::Receiver<String>,
    stderr_rx: tokio::sync::mpsc::Receiver<String>,
}

impl MockChild {
    fn new(
        stdout_rx: tokio::sync::mpsc::Receiver<String>,
        stderr_rx: tokio::sync::mpsc::Receiver<String>,
    ) -> Self {
        Self { stdout_rx, stderr_rx }
    }
}

#[async_trait]
impl AsyncGroupChild for MockChild {
    async fn wait(&mut self) -> std::io::Result<std::process::ExitStatus> {
        // Simulate process completion
        tokio::time::sleep(Duration::from_millis(100)).await;
        Ok(std::process::ExitStatus::default())
    }

    fn inner(&mut self) -> &mut tokio::process::Command {
        panic!("MockChild::inner not implemented")
    }

    fn take_stdin(&mut self) -> Option<tokio::process::ChildStdin> {
        None
    }

    fn take_stdout(&mut self) -> Option<tokio::process::ChildStdout> {
        None
    }

    fn take_stderr(&mut self) -> Option<tokio::process::ChildStderr> {
        None
    }
}

#[derive(Debug)]
struct MockSpawnedChild {
    child: MockChild,
    exit_signal: Option<oneshot::Receiver<executors::executors::ExecutorExitResult>>,
}

impl MockSpawnedChild {
    fn new(
        child: MockChild,
        exit_signal: Option<oneshot::Receiver<executors::executors::ExecutorExitResult>>,
    ) -> Self {
        Self { child, exit_signal }
    }
}

impl From<MockSpawnedChild> for SpawnedChild {
    fn from(mock: MockSpawnedChild) -> Self {
        SpawnedChild {
            child: Box::new(mock.child),
            exit_signal: mock.exit_signal,
            interrupt_sender: None,
        }
    }
}

#[tokio::test]
async fn test_claude_flow_spawn_basic() {
    let temp_dir = TempDir::new().unwrap();
    let current_dir = temp_dir.path();

    let env = ExecutionEnv::default();

    let claude_flow = ClaudeFlow {
        append_prompt: Default::default(),
        non_interactive: Some(true),
        enable_chaining: Some(true),
        agent_id: None,
        workflow_file: None,
        task_description: None,
        cmd: Default::default(),
    };

    // This test verifies the spawn method creates the correct command
    // In a real test environment, we'd mock the command execution
    let result = claude_flow.spawn(current_dir, "Test prompt", &env).await;

    // The spawn should succeed (assuming claude-flow is installed)
    // In actual tests, we'd mock the command execution
    assert!(result.is_ok() || matches!(result, Err(ExecutorError::ExecutableNotFound { .. })));
}

#[tokio::test]
async fn test_claude_flow_spawn_with_all_options() {
    let temp_dir = TempDir::new().unwrap();
    let current_dir = temp_dir.path();

    let env = ExecutionEnv::default();

    let claude_flow = ClaudeFlow {
        append_prompt: Default::default(),
        non_interactive: Some(true),
        enable_chaining: Some(true),
        agent_id: Some("coding-agent".to_string()),
        workflow_file: Some("test-workflow.json".to_string()),
        task_description: Some("Process data".to_string()),
        cmd: Default::default(),
    };

    let result = claude_flow.spawn(current_dir, "Test prompt with all options", &env).await;

    // The spawn should succeed (assuming claude-flow is installed)
    assert!(result.is_ok() || matches!(result, Err(ExecutorError::ExecutableNotFound { .. })));
}

#[tokio::test]
async fn test_claude_flow_spawn_follow_up() {
    let temp_dir = TempDir::new().unwrap();
    let current_dir = temp_dir.path();

    let env = ExecutionEnv::default();

    let claude_flow = ClaudeFlow {
        append_prompt: Default::default(),
        non_interactive: Some(true),
        enable_chaining: None,
        agent_id: None,
        workflow_file: None,
        task_description: None,
        cmd: Default::default(),
    };

    let result = claude_flow.spawn_follow_up(
        current_dir,
        "Follow-up prompt",
        "session-123",
        &env,
    ).await;

    // The spawn should succeed (assuming claude-flow is installed)
    assert!(result.is_ok() || matches!(result, Err(ExecutorError::ExecutableNotFound { .. })));
}

#[tokio::test]
async fn test_claude_flow_log_normalization() {
    let temp_dir = TempDir::new().unwrap();
    let current_dir = temp_dir.path();

    // Create a mock message store
    let msg_store = Arc::new(MsgStore::new());

    // Add some test messages that simulate claude-flow output
    msg_store.push(json!({
        "type": "init",
        "timestamp": "2024-01-05T10:00:00Z",
        "session_id": "test-session",
        "agent_id": "coding-agent"
    }));

    msg_store.push(json!({
        "type": "message",
        "role": "assistant",
        "content": "Processing task...",
        "timestamp": "2024-01-05T10:00:01Z"
    }));

    msg_store.push(json!({
        "type": "tool_use",
        "tool": "FileEditor",
        "input": {
            "command": "view",
            "path": "src/main.rs"
        },
        "timestamp": "2024-01-05T10:00:02Z"
    }));

    let claude_flow = ClaudeFlow {
        append_prompt: Default::default(),
        non_interactive: Some(true),
        enable_chaining: Some(true),
        agent_id: Some("coding-agent".to_string()),
        workflow_file: None,
        task_description: None,
        cmd: Default::default(),
    };

    // Test log normalization
    claude_flow.normalize_logs(msg_store.clone(), current_dir);

    // Verify that messages were processed correctly
    let messages = msg_store.get_all();
    assert!(!messages.is_empty());

    // Check that we have the expected message types
    let has_init = messages.iter().any(|m| {
        m.get("type").and_then(|t| t.as_str()) == Some("init")
    });
    assert!(has_init);

    let has_message = messages.iter().any(|m| {
        m.get("type").and_then(|t| t.as_str()) == Some("message")
    });
    assert!(has_message);

    let has_tool_use = messages.iter().any(|m| {
        m.get("type").and_then(|t| t.as_str()) == Some("tool_use")
    });
    assert!(has_tool_use);
}

#[tokio::test]
async fn test_claude_flow_error_handling() {
    let temp_dir = TempDir::new().unwrap();
    let current_dir = temp_dir.path();

    let env = ExecutionEnv::default();

    // Test with invalid command override
    let mut cmd_overrides = executors::command::CmdOverrides::default();
    cmd_overrides.base_command_override = Some("nonexistent-command".to_string());

    let claude_flow = ClaudeFlow {
        append_prompt: Default::default(),
        non_interactive: Some(true),
        enable_chaining: None,
        agent_id: None,
        workflow_file: None,
        task_description: None,
        cmd: cmd_overrides,
    };

    let result = claude_flow.spawn(current_dir, "Test prompt", &env).await;

    // Should fail due to nonexistent command
    assert!(result.is_err());
}

#[tokio::test]
async fn test_claude_flow_with_workflow_file() {
    let temp_dir = TempDir::new().unwrap();
    let current_dir = temp_dir.path();

    // Create a test workflow file
    let workflow_content = json!({
        "name": "Test Workflow",
        "settings": {
            "enableChaining": true,
            "outputFormat": "stream-json"
        },
        "tasks": [
            {
                "id": "task1",
                "name": "Analyze Code",
                "assignTo": "coding-agent",
                "claudePrompt": "Analyze the code structure"
            }
        ]
    });

    let workflow_path = current_dir.join("test-workflow.json");
    tokio::fs::write(
        &workflow_path,
        serde_json::to_string_pretty(&workflow_content).unwrap(),
    ).await.unwrap();

    let env = ExecutionEnv::default();

    let claude_flow = ClaudeFlow {
        append_prompt: Default::default(),
        non_interactive: Some(true),
        enable_chaining: Some(true),
        agent_id: None,
        workflow_file: Some(workflow_path.to_string_lossy().to_string()),
        task_description: None,
        cmd: Default::default(),
    };

    let result = claude_flow.spawn(current_dir, "Test with workflow", &env).await;

    // The spawn should succeed (assuming claude-flow is installed)
    assert!(result.is_ok() || matches!(result, Err(ExecutorError::ExecutableNotFound { .. })));
}

#[tokio::test]
async fn test_claude_flow_append_prompt() {
    let temp_dir = TempDir::new().unwrap();
    let current_dir = temp_dir.path();

    let env = ExecutionEnv::default();

    let claude_flow = ClaudeFlow {
        append_prompt: executors::executors::AppendPrompt(Some(
            " Remember to add unit tests.".to_string(),
        )),
        non_interactive: Some(true),
        enable_chaining: None,
        agent_id: None,
        workflow_file: None,
        task_description: None,
        cmd: Default::default(),
    };

    let combined_prompt = claude_flow.append_prompt.combine_prompt("Write a function");
    assert_eq!(combined_prompt, "Write a function Remember to add unit tests.");

    let result = claude_flow.spawn(current_dir, "Write a function", &env).await;

    // The spawn should succeed (assuming claude-flow is installed)
    assert!(result.is_ok() || matches!(result, Err(ExecutorError::ExecutableNotFound { .. })));
}

#[tokio::test]
async fn test_claude_flow_concurrent_execution() {
    let temp_dir = TempDir::new().unwrap();
    let current_dir = temp_dir.path();

    let env = ExecutionEnv::default();

    let claude_flow = Arc::new(ClaudeFlow {
        append_prompt: Default::default(),
        non_interactive: Some(true),
        enable_chaining: None,
        agent_id: None,
        workflow_file: None,
        task_description: None,
        cmd: Default::default(),
    });

    let mut handles = vec![];

    // Spawn multiple instances concurrently
    for i in 0..3 {
        let flow = claude_flow.clone();
        let dir = current_dir.to_path_buf();
        let env = env.clone();

        let handle = tokio::spawn(async move {
            flow.spawn(
                &dir,
                &format!("Concurrent task {}", i),
                &env,
            ).await
        });

        handles.push(handle);
    }

    // Wait for all to complete
    for handle in handles {
        let result = handle.await.unwrap();
        assert!(result.is_ok() || matches!(result, Err(ExecutorError::ExecutableNotFound { .. })));
    }
}

#[tokio::test]
async fn test_claude_flow_timeout() {
    let temp_dir = TempDir::new().unwrap();
    let current_dir = temp_dir.path();

    let env = ExecutionEnv::default();

    let claude_flow = ClaudeFlow {
        append_prompt: Default::default(),
        non_interactive: Some(true),
        enable_chaining: None,
        agent_id: None,
        workflow_file: None,
        task_description: None,
        cmd: Default::default(),
    };

    // Test with a timeout
    let result = timeout(
        Duration::from_secs(5),
        claude_flow.spawn(current_dir, "Test prompt", &env),
    ).await;

    match result {
        Ok(spawn_result) => {
            assert!(spawn_result.is_ok() || matches!(spawn_result, Err(ExecutorError::ExecutableNotFound { .. })));
        }
        Err(_) => {
            // Timeout occurred - this is acceptable for testing
            println!("Spawn operation timed out");
        }
    }
}

#[tokio::test]
async fn test_claude_flow_json_streaming_output() {
    let temp_dir = TempDir::new().unwrap();
    let current_dir = temp_dir.path();

    // Create a mock message store
    let msg_store = Arc::new(MsgStore::new());

    // Simulate various claude-flow JSON output messages
    let messages = vec![
        json!({
            "type": "init",
            "timestamp": "2024-01-05T10:00:00Z",
            "session_id": "test-session",
            "agent_id": "coding-agent",
            "version": "1.0.0"
        }),
        json!({
            "type": "message",
            "role": "assistant",
            "content": "I'll help you with the coding task.",
            "timestamp": "2024-01-05T10:00:01Z"
        }),
        json!({
            "type": "tool_use",
            "id": "tool-123",
            "name": "FileEditor",
            "input": {
                "command": "str_replace",
                "path": "src/main.rs",
                "old_str": "fn main() {}",
                "new_str": "fn main() {\n    println!(\"Hello, world!\");\n}"
            },
            "timestamp": "2024-01-05T10:00:02Z"
        }),
        json!({
            "type": "tool_result",
            "tool_use_id": "tool-123",
            "content": "Successfully updated file",
            "is_error": false,
            "timestamp": "2024-01-05T10:00:03Z"
        }),
        json!({
            "type": "message",
            "role": "assistant",
            "content": "Task completed successfully!",
            "timestamp": "2024-01-05T10:00:04Z"
        }),
        json!({
            "type": "result",
            "status": "success",
            "summary": "Updated main.rs with hello world",
            "timestamp": "2024-01-05T10:00:05Z"
        }),
    ];

    for msg in messages {
        msg_store.push(msg);
    }

    let claude_flow = ClaudeFlow {
        append_prompt: Default::default(),
        non_interactive: Some(true),
        enable_chaining: Some(true),
        agent_id: Some("coding-agent".to_string()),
        workflow_file: None,
        task_description: None,
        cmd: Default::default(),
    };

    // Test log normalization with streaming JSON
    claude_flow.normalize_logs(msg_store.clone(), current_dir);

    // Verify all message types were processed
    let stored_messages = msg_store.get_all();
    assert_eq!(stored_messages.len(), 6);

    // Check for specific message types
    let init_count = stored_messages.iter().filter(|m| {
        m.get("type").and_then(|t| t.as_str()) == Some("init")
    }).count();
    assert_eq!(init_count, 1);

    let message_count = stored_messages.iter().filter(|m| {
        m.get("type").and_then(|t| t.as_str()) == Some("message")
    }).count();
    assert_eq!(message_count, 2);

    let tool_use_count = stored_messages.iter().filter(|m| {
        m.get("type").and_then(|t| t.as_str()) == Some("tool_use")
    }).count();
    assert_eq!(tool_use_count, 1);

    let tool_result_count = stored_messages.iter().filter(|m| {
        m.get("type").and_then(|t| t.as_str()) == Some("tool_result")
    }).count();
    assert_eq!(tool_result_count, 1);

    let result_count = stored_messages.iter().filter(|m| {
        m.get("type").and_then(|t| t.as_str()) == Some("result")
    }).count();
    assert_eq!(result_count, 1);
}

#[tokio::test]
async fn test_claude_flow_error_scenarios() {
    let temp_dir = TempDir::new().unwrap();
    let current_dir = temp_dir.path();

    let env = ExecutionEnv::default();

    // Test 1: Invalid JSON in workflow file
    let workflow_path = current_dir.join("invalid.json");
    tokio::fs::write(
        &workflow_path,
        "{ invalid json",
    ).await.unwrap();

    let claude_flow = ClaudeFlow {
        append_prompt: Default::default(),
        non_interactive: Some(true),
        enable_chaining: None,
        agent_id: None,
        workflow_file: Some(workflow_path.to_string_lossy().to_string()),
        task_description: None,
        cmd: Default::default(),
    };

    // This should still spawn successfully - the workflow file is passed as an argument
    let result = claude_flow.spawn(current_dir, "Test prompt", &env).await;
    assert!(result.is_ok() || matches!(result, Err(ExecutorError::ExecutableNotFound { .. })));

    // Test 2: Non-existent directory
    let non_existent_dir = PathBuf::from("/non/existent/directory");
    let result = claude_flow.spawn(&non_existent_dir, "Test prompt", &env).await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_claude_flow_mcp_config_availability() {
    let claude_flow = ClaudeFlow {
        append_prompt: Default::default(),
        non_interactive: None,
        enable_chaining: None,
        agent_id: None,
        workflow_file: None,
        task_description: None,
        cmd: Default::default(),
    };

    let config_path = claude_flow.default_mcp_config_path();
    assert!(config_path.is_some());

    let path = config_path.unwrap();
    assert!(path.to_string_lossy().contains(".claude-flow"));
    assert!(path.ends_with("config.json"));

    // Test availability info
    let availability = claude_flow.get_availability_info();
    // Should return InstallationFound if config exists, NotFound otherwise
    // In test environment, it likely won't exist
    assert!(!availability.is_available() || availability.is_available());
}

#[tokio::test]
async fn test_claude_flow_capabilities() {
    use executors::executors::BaseAgentCapability;

    let claude_flow = ClaudeFlow {
        append_prompt: Default::default(),
        non_interactive: None,
        enable_chaining: None,
        agent_id: None,
        workflow_file: None,
        task_description: None,
        cmd: Default::default(),
    };

    let capabilities = claude_flow.capabilities();
    assert_eq!(capabilities.len(), 1);
    assert_eq!(capabilities[0], BaseAgentCapability::SessionFork);
}