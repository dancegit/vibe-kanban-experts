use std::{path::Path, process::Stdio, sync::Arc};

use async_trait::async_trait;
use command_group::AsyncCommandGroup;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use tokio::{io::AsyncWriteExt, process::Command};
use ts_rs::TS;
use workspace_utils::msg_store::MsgStore;

use crate::{
    command::{CmdOverrides, CommandBuilder, apply_overrides},
    env::ExecutionEnv,
    executors::{
        AppendPrompt, AvailabilityInfo, ExecutorError, SpawnedChild, StandardCodingAgentExecutor,
        claude::{ClaudeLogProcessor, HistoryStrategy},
    },
    logs::{stderr_processor::normalize_stderr_logs, utils::EntryIndexProvider},
};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, TS, JsonSchema)]
pub struct ClaudeFlow {
    #[serde(default)]
    pub append_prompt: AppendPrompt,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[schemars(
        title = "Non-interactive Mode",
        description = "Run in non-interactive mode for automation"
    )]
    pub non_interactive: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[schemars(
        title = "Enable Chaining",
        description = "Enable stream chaining between agents"
    )]
    pub enable_chaining: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[schemars(
        title = "Agent ID",
        description = "Specific agent to run"
    )]
    pub agent_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[schemars(
        title = "Workflow File",
        description = "Path to workflow configuration file"
    )]
    pub workflow_file: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[schemars(
        title = "Task Description",
        description = "Task description for automation commands"
    )]
    pub task_description: Option<String>,
    #[serde(flatten)]
    pub cmd: CmdOverrides,
}

impl ClaudeFlow {
    fn build_command_builder(&self) -> CommandBuilder {
        // Base command - use claude-flow automation for non-interactive mode
        let base_cmd = if self.non_interactive.unwrap_or(false) {
            "npx -y claude-flow automation"
        } else {
            "npx -y claude-flow"
        };

        let mut builder = CommandBuilder::new(base_cmd)
            .params(["--output-format", "stream-json"])
            .extend_params(["--input-format", "stream-json"]);

        // Add chaining option
        if self.enable_chaining.unwrap_or(false) {
            builder = builder.extend_params(["--chaining"]);
        }

        // Add agent ID if specified
        if let Some(agent_id) = &self.agent_id {
            builder = builder.extend_params(["--agent", agent_id]);
        }

        // Add workflow file if specified
        if let Some(workflow) = &self.workflow_file {
            builder = builder.extend_params(["--workflow", workflow]);
        }

        // Add task description for automation mode
        if let Some(task) = &self.task_description {
            builder = builder.extend_params(["--task", task]);
        }

        apply_overrides(builder, &self.cmd)
    }
}

#[async_trait]
impl StandardCodingAgentExecutor for ClaudeFlow {
    async fn spawn(
        &self,
        current_dir: &Path,
        prompt: &str,
        env: &ExecutionEnv,
    ) -> Result<SpawnedChild, ExecutorError> {
        let command_parts = self.build_command_builder().build_initial()?;
        let (executable_path, args) = command_parts.into_resolved().await?;

        let combined_prompt = self.append_prompt.combine_prompt(prompt);

        let mut command = Command::new(executable_path);
        command
            .kill_on_drop(true)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .current_dir(current_dir)
            .args(&args);

        env.clone()
            .with_profile(&self.cmd)
            .apply_to_command(&mut command);

        let mut child = command.group_spawn()?;

        // Feed the prompt in, then close the pipe so claude-flow sees EOF
        if let Some(mut stdin) = child.inner().stdin.take() {
            stdin.write_all(combined_prompt.as_bytes()).await?;
            stdin.shutdown().await?;
        }

        Ok(child.into())
    }

    async fn spawn_follow_up(
        &self,
        current_dir: &Path,
        prompt: &str,
        session_id: &str,
        env: &ExecutionEnv,
    ) -> Result<SpawnedChild, ExecutorError> {
        // Claude-flow doesn't support follow-up with session_id like ClaudeCode
        // We need to use a different approach for continuing conversations
        // For now, we'll spawn a new process with the session context

        let command_parts = self.build_command_builder().build_follow_up(&[
            "--resume".to_string(),
            session_id.to_string(),
        ])?;
        let (executable_path, args) = command_parts.into_resolved().await?;

        let combined_prompt = self.append_prompt.combine_prompt(prompt);

        let mut command = Command::new(executable_path);
        command
            .kill_on_drop(true)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .current_dir(current_dir)
            .args(&args);

        env.clone()
            .with_profile(&self.cmd)
            .apply_to_command(&mut command);

        let mut child = command.group_spawn()?;

        // Feed the prompt in, then close the pipe so claude-flow sees EOF
        if let Some(mut stdin) = child.inner().stdin.take() {
            stdin.write_all(combined_prompt.as_bytes()).await?;
            stdin.shutdown().await?;
        }

        Ok(child.into())
    }

    fn normalize_logs(&self, msg_store: Arc<MsgStore>, current_dir: &Path) {
        let entry_index_provider = EntryIndexProvider::start_from(&msg_store);

        // Process stdout logs (ClaudeFlow's stream JSON output) using Claude's log processor
        // ClaudeFlow outputs similar stream JSON format
        ClaudeLogProcessor::process_logs(
            msg_store.clone(),
            current_dir,
            entry_index_provider.clone(),
            HistoryStrategy::Default,
        );

        // Process stderr logs using the standard stderr processor
        normalize_stderr_logs(msg_store, entry_index_provider);
    }

    // MCP configuration methods
    fn default_mcp_config_path(&self) -> Option<std::path::PathBuf> {
        dirs::home_dir().map(|home| home.join(".claude-flow").join("config.json"))
    }

    fn get_availability_info(&self) -> AvailabilityInfo {
        let config_file_path = self.default_mcp_config_path();

        if let Some(path) = config_file_path
            && let Some(timestamp) = std::fs::metadata(&path)
                .ok()
                .and_then(|m| m.modified().ok())
                .and_then(|modified| modified.duration_since(std::time::UNIX_EPOCH).ok())
                .map(|d| d.as_secs() as i64)
        {
            return AvailabilityInfo::LoginDetected {
                last_auth_timestamp: timestamp,
            };
        }
        AvailabilityInfo::NotFound
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_claude_flow_deserialization() {
        let json = r#"{
            "append_prompt": "Additional context",
            "non_interactive": true,
            "enable_chaining": true,
            "agent_id": "coding-agent",
            "workflow_file": "workflow.json",
            "task_description": "Test task"
        }"#;

        let result: Result<ClaudeFlow, _> = serde_json::from_str(json);
        assert!(result.is_ok());

        let flow = result.unwrap();
        assert_eq!(flow.append_prompt.0, Some("Additional context".to_string()));
        assert_eq!(flow.non_interactive, Some(true));
        assert_eq!(flow.enable_chaining, Some(true));
        assert_eq!(flow.agent_id, Some("coding-agent".to_string()));
        assert_eq!(flow.workflow_file, Some("workflow.json".to_string()));
        assert_eq!(flow.task_description, Some("Test task".to_string()));
    }

    #[test]
    fn test_claude_flow_minimal_config() {
        let json = r#"{}"#;

        let result: Result<ClaudeFlow, _> = serde_json::from_str(json);
        assert!(result.is_ok());

        let flow = result.unwrap();
        assert!(flow.append_prompt.0.is_none());
        assert_eq!(flow.non_interactive, None);
        assert_eq!(flow.enable_chaining, None);
        assert_eq!(flow.agent_id, None);
        assert_eq!(flow.workflow_file, None);
        assert_eq!(flow.task_description, None);
    }

    #[test]
    fn test_claude_flow_command_builder_non_interactive() {
        let flow = ClaudeFlow {
            append_prompt: AppendPrompt(None),
            non_interactive: Some(true),
            enable_chaining: Some(true),
            agent_id: Some("test-agent".to_string()),
            workflow_file: None,
            task_description: None,
            cmd: CmdOverrides::default(),
        };

        let builder = flow.build_command_builder();
        let cmd_str = format!("{}", builder);

        assert!(cmd_str.contains("npx -y claude-flow automation"));
        assert!(cmd_str.contains("--output-format stream-json"));
        assert!(cmd_str.contains("--input-format stream-json"));
        assert!(cmd_str.contains("--chaining"));
        assert!(cmd_str.contains("--agent test-agent"));
    }

    #[test]
    fn test_claude_flow_command_builder_interactive() {
        let flow = ClaudeFlow {
            append_prompt: AppendPrompt(None),
            non_interactive: Some(false),
            enable_chaining: None,
            agent_id: None,
            workflow_file: Some("test.json".to_string()),
            task_description: Some("my task".to_string()),
            cmd: CmdOverrides::default(),
        };

        let builder = flow.build_command_builder();
        let cmd_str = format!("{}", builder);

        assert!(cmd_str.contains("npx -y claude-flow"));
        assert!(cmd_str.contains("--output-format stream-json"));
        assert!(cmd_str.contains("--input-format stream-json"));
        assert!(cmd_str.contains("--workflow test.json"));
        assert!(cmd_str.contains("--task my task"));
    }

    #[test]
    fn test_claude_flow_command_builder_default() {
        let flow = ClaudeFlow {
            append_prompt: AppendPrompt(None),
            non_interactive: None,
            enable_chaining: None,
            agent_id: None,
            workflow_file: None,
            task_description: None,
            cmd: CmdOverrides::default(),
        };

        let builder = flow.build_command_builder();
        let cmd_str = format!("{}", builder);

        assert!(cmd_str.contains("npx -y claude-flow"));
        assert!(cmd_str.contains("--output-format stream-json"));
        assert!(cmd_str.contains("--input-format stream-json"));
    }

    #[test]
    fn test_append_prompt_combination() {
        let flow = ClaudeFlow {
            append_prompt: AppendPrompt(Some(" Extra context".to_string())),
            non_interactive: None,
            enable_chaining: None,
            agent_id: None,
            workflow_file: None,
            task_description: None,
            cmd: CmdOverrides::default(),
        };

        let combined = flow.append_prompt.combine_prompt("Base prompt");
        assert_eq!(combined, "Base prompt Extra context");
    }

    #[test]
    fn test_append_prompt_none() {
        let flow = ClaudeFlow {
            append_prompt: AppendPrompt(None),
            non_interactive: None,
            enable_chaining: None,
            agent_id: None,
            workflow_file: None,
            task_description: None,
            cmd: CmdOverrides::default(),
        };

        let combined = flow.append_prompt.combine_prompt("Base prompt");
        assert_eq!(combined, "Base prompt");
    }

    #[test]
    fn test_default_mcp_config_path() {
        let flow = ClaudeFlow {
            append_prompt: AppendPrompt(None),
            non_interactive: None,
            enable_chaining: None,
            agent_id: None,
            workflow_file: None,
            task_description: None,
            cmd: CmdOverrides::default(),
        };

        let config_path = flow.default_mcp_config_path();
        assert!(config_path.is_some());

        let path = config_path.unwrap();
        assert!(path.to_string_lossy().contains(".claude-flow"));
        assert!(path.ends_with("config.json"));
    }

    #[test]
    fn test_claude_flow_serialization_roundtrip() {
        let original = ClaudeFlow {
            append_prompt: AppendPrompt(Some("test".to_string())),
            non_interactive: Some(true),
            enable_chaining: Some(false),
            agent_id: Some("agent1".to_string()),
            workflow_file: Some("workflow.json".to_string()),
            task_description: Some("test task".to_string()),
            cmd: CmdOverrides::default(),
        };

        let serialized = serde_json::to_string(&original).unwrap();
        let deserialized: ClaudeFlow = serde_json::from_str(&serialized).unwrap();

        assert_eq!(original.append_prompt.0, deserialized.append_prompt.0);
        assert_eq!(original.non_interactive, deserialized.non_interactive);
        assert_eq!(original.enable_chaining, deserialized.enable_chaining);
        assert_eq!(original.agent_id, deserialized.agent_id);
        assert_eq!(original.workflow_file, deserialized.workflow_file);
        assert_eq!(original.task_description, deserialized.task_description);
    }

    #[test]
    fn test_command_builder_with_all_options() {
        let flow = ClaudeFlow {
            append_prompt: AppendPrompt(Some(" Additional prompt".to_string())),
            non_interactive: Some(true),
            enable_chaining: Some(true),
            agent_id: Some("swarm-coordinator".to_string()),
            workflow_file: Some("complex-workflow.json".to_string()),
            task_description: Some("Complex multi-agent task".to_string()),
            cmd: CmdOverrides {
                base_command_override: Some("custom-claude-flow".to_string()),
                additional_params: Some(vec!["--param1".to_string(), "--param2".to_string()]),
                env: Some(std::collections::HashMap::from([
                    ("ENV_VAR1".to_string(), "value1".to_string()),
                    ("ENV_VAR2".to_string(), "value2".to_string()),
                ])),
            },
        };

        let builder = flow.build_command_builder();
        let cmd_str = format!("{}", builder);

        // Check base command override
        assert!(cmd_str.contains("custom-claude-flow"));

        // Check all options are included
        assert!(cmd_str.contains("--output-format stream-json"));
        assert!(cmd_str.contains("--input-format stream-json"));
        assert!(cmd_str.contains("--chaining"));
        assert!(cmd_str.contains("--agent swarm-coordinator"));
        assert!(cmd_str.contains("--workflow complex-workflow.json"));
        assert!(cmd_str.contains("--task Complex multi-agent task"));

        // Check additional params
        assert!(cmd_str.contains("--param1"));
        assert!(cmd_str.contains("--param2"));
    }

    #[test]
    fn test_command_builder_disable_chaining() {
        let flow = ClaudeFlow {
            append_prompt: AppendPrompt(None),
            non_interactive: Some(true),
            enable_chaining: Some(false), // Explicitly disabled
            agent_id: None,
            workflow_file: None,
            task_description: None,
            cmd: CmdOverrides::default(),
        };

        let builder = flow.build_command_builder();
        let cmd_str = format!("{}", builder);

        // Should NOT contain chaining when explicitly disabled
        assert!(!cmd_str.contains("--chaining"));
        assert!(cmd_str.contains("--output-format stream-json"));
    }

    #[test]
    fn test_get_availability_info_with_config_file() {
        use std::fs;
        use tempfile::TempDir;

        // Create a temporary directory with a config file
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join(".claude-flow").join("config.json");

        // Create the directory and file
        fs::create_dir_all(config_path.parent().unwrap()).unwrap();
        fs::write(&config_path, r#"{"auth": "test"}"#).unwrap();

        // Mock the home directory by temporarily setting an environment variable
        std::env::set_var("HOME", temp_dir.path().to_str().unwrap());

        let flow = ClaudeFlow {
            append_prompt: AppendPrompt(None),
            non_interactive: None,
            enable_chaining: None,
            agent_id: None,
            workflow_file: None,
            task_description: None,
            cmd: CmdOverrides::default(),
        };

        let availability = flow.get_availability_info();

        // Clean up
        std::env::remove_var("HOME");

        // Should detect the config file
        match availability {
            AvailabilityInfo::LoginDetected { .. } | AvailabilityInfo::InstallationFound => {
                // Success - config file was detected
            }
            AvailabilityInfo::NotFound => {
                panic!("Expected config file to be detected");
            }
        }
    }

    #[test]
    fn test_get_availability_info_without_config() {
        let flow = ClaudeFlow {
            append_prompt: AppendPrompt(None),
            non_interactive: None,
            enable_chaining: None,
            agent_id: None,
            workflow_file: None,
            task_description: None,
            cmd: CmdOverrides::default(),
        };

        let availability = flow.get_availability_info();

        // Without config file, should return NotFound
        assert!(matches!(availability, AvailabilityInfo::NotFound));
    }

    #[test]
    fn test_ts_rs_type_derivation() {
        // Test that TypeScript types can be derived
        let flow = ClaudeFlow::default();

        // This should compile without errors if TS derivation works
        let _ts_type = std::any::type_name::<ClaudeFlow>();

        // Test serialization for TS
        let serialized = serde_json::to_string(&flow).unwrap();
        let deserialized: ClaudeFlow = serde_json::from_str(&serialized).unwrap();

        assert_eq!(flow.append_prompt.0, deserialized.append_prompt.0);
    }

    #[test]
    fn test_schemars_json_schema() {
        // Test that JSON schema can be generated
        let flow = ClaudeFlow::default();

        // This should compile without errors if JsonSchema derivation works
        let _schema = schemars::schema_for!(ClaudeFlow);

        // Verify that the schema can be generated
        assert!(_schema.title.is_some());
    }

    #[test]
    fn test_empty_string_handling() {
        let flow = ClaudeFlow {
            append_prompt: AppendPrompt(Some("".to_string())),
            non_interactive: Some(true),
            enable_chaining: Some(false),
            agent_id: Some("".to_string()),
            workflow_file: Some("".to_string()),
            task_description: Some("".to_string()),
            cmd: CmdOverrides::default(),
        };

        // Test that empty strings are handled properly
        assert_eq!(flow.append_prompt.0, Some("".to_string()));
        assert_eq!(flow.agent_id, Some("".to_string()));
        assert_eq!(flow.workflow_file, Some("".to_string()));
        assert_eq!(flow.task_description, Some("".to_string()));

        let builder = flow.build_command_builder();
        let cmd_str = format!("{}", builder);

        // Empty strings should still produce valid command structure
        assert!(cmd_str.contains("--output-format stream-json"));
    }

    #[test]
    fn test_special_characters_in_config() {
        let flow = ClaudeFlow {
            append_prompt: AppendPrompt(Some("Special chars: <>&\"'".to_string())),
            non_interactive: Some(true),
            enable_chaining: Some(true),
            agent_id: Some("agent-with-dashes_and_underscores".to_string()),
            workflow_file: Some("/path/to/workflow.json".to_string()),
            task_description: Some("Task with \"quotes\" and 'apostrophes'".to_string()),
            cmd: CmdOverrides::default(),
        };

        // Test serialization/deserialization with special characters
        let serialized = serde_json::to_string(&flow).unwrap();
        let deserialized: ClaudeFlow = serde_json::from_str(&serialized).unwrap();

        assert_eq!(flow.append_prompt.0, deserialized.append_prompt.0);
        assert_eq!(flow.agent_id, deserialized.agent_id);
        assert_eq!(flow.workflow_file, deserialized.workflow_file);
        assert_eq!(flow.task_description, deserialized.task_description);
    }

    #[test]
    fn test_derivative_traits() {
        let flow1 = ClaudeFlow {
            append_prompt: AppendPrompt(Some("test".to_string())),
            non_interactive: Some(true),
            enable_chaining: Some(false),
            agent_id: Some("agent1".to_string()),
            workflow_file: Some("workflow.json".to_string()),
            task_description: Some("task".to_string()),
            cmd: CmdOverrides::default(),
        };

        let flow2 = ClaudeFlow {
            append_prompt: AppendPrompt(Some("test".to_string())),
            non_interactive: Some(true),
            enable_chaining: Some(false),
            agent_id: Some("agent1".to_string()),
            workflow_file: Some("workflow.json".to_string()),
            task_description: Some("task".to_string()),
            cmd: CmdOverrides::default(),
        };

        let flow3 = ClaudeFlow {
            append_prompt: AppendPrompt(Some("different".to_string())),
            non_interactive: Some(true),
            enable_chaining: Some(false),
            agent_id: Some("agent1".to_string()),
            workflow_file: Some("workflow.json".to_string()),
            task_description: Some("task".to_string()),
            cmd: CmdOverrides::default(),
        };

        // Test PartialEq
        assert_eq!(flow1, flow2);
        assert_ne!(flow1, flow3);

        // Test Debug (should compile)
        let debug_str = format!("{:?}", flow1);
        assert!(debug_str.contains("ClaudeFlow"));

        // Test Clone
        let cloned = flow1.clone();
        assert_eq!(flow1, cloned);
    }
}
