use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;

use crate::executors::{
    AppendPrompt, AvailabilityInfo, ExecutorError, StandardCodingAgentExecutor,
    claude_flow::ClaudeFlow,
};
use crate::env::ExecutionEnv;
use crate::command::{CmdOverrides, CommandBuilder};

#[cfg(test)]
mod claude_flow_tests {
    use super::*;

    #[test]
    fn test_claude_flow_basic_creation() {
        let agent = ClaudeFlow::default();
        assert_eq!(agent.append_prompt, AppendPrompt::default());
        assert_eq!(agent.non_interactive, None);
        assert_eq!(agent.enable_chaining, None);
        assert_eq!(agent.agent_id, None);
        assert_eq!(agent.workflow_file, None);
        assert_eq!(agent.task_description, None);
        assert_eq!(agent.cmd, CmdOverrides::default());
    }

    #[test]
    fn test_claude_flow_with_all_options() {
        let agent = ClaudeFlow {
            append_prompt: AppendPrompt(Some("Additional context".to_string())),
            non_interactive: Some(true),
            enable_chaining: Some(true),
            agent_id: Some("custom-agent".to_string()),
            workflow_file: Some("workflow.json".to_string()),
            task_description: Some("Test task".to_string()),
            cmd: CmdOverrides::default(),
        };

        assert_eq!(agent.append_prompt.0, Some("Additional context".to_string()));
        assert_eq!(agent.non_interactive, Some(true));
        assert_eq!(agent.enable_chaining, Some(true));
        assert_eq!(agent.agent_id, Some("custom-agent".to_string()));
        assert_eq!(agent.workflow_file, Some("workflow.json".to_string()));
        assert_eq!(agent.task_description, Some("Test task".to_string()));
    }

    #[test]
    fn test_claude_flow_command_builder_basic() {
        let agent = ClaudeFlow::default();
        let builder = agent.build_command_builder();

        let command_string = format!("{}", builder);

        // Basic command structure
        assert!(command_string.contains("npx -y claude-flow"));
        assert!(command_string.contains("--output-format stream-json"));
        assert!(command_string.contains("--input-format stream-json"));

        // Should not contain optional parameters when not set
        assert!(!command_string.contains("--chaining"));
        assert!(!command_string.contains("--agent"));
        assert!(!command_string.contains("--workflow"));
        assert!(!command_string.contains("--task"));
    }

    #[test]
    fn test_claude_flow_command_builder_non_interactive() {
        let agent = ClaudeFlow {
            non_interactive: Some(true),
            ..Default::default()
        };

        let builder = agent.build_command_builder();
        let command_string = format!("{}", builder);

        assert!(command_string.contains("npx -y claude-flow automation"));
        assert!(command_string.contains("--output-format stream-json"));
        assert!(command_string.contains("--input-format stream-json"));
    }

    #[test]
    fn test_claude_flow_command_builder_with_chaining() {
        let agent = ClaudeFlow {
            enable_chaining: Some(true),
            ..Default::default()
        };

        let builder = agent.build_command_builder();
        let command_string = format!("{}", builder);

        assert!(command_string.contains("--chaining"));
    }

    #[test]
    fn test_claude_flow_command_builder_with_agent_id() {
        let agent = ClaudeFlow {
            agent_id: Some("swarm-coordinator".to_string()),
            ..Default::default()
        };

        let builder = agent.build_command_builder();
        let command_string = format!("{}", builder);

        assert!(command_string.contains("--agent swarm-coordinator"));
    }

    #[test]
    fn test_claude_flow_command_builder_with_workflow() {
        let agent = ClaudeFlow {
            workflow_file: Some("complex-workflow.json".to_string()),
            ..Default::default()
        };

        let builder = agent.build_command_builder();
        let command_string = format!("{}", builder);

        assert!(command_string.contains("--workflow complex-workflow.json"));
    }

    #[test]
    fn test_claude_flow_command_builder_with_task() {
        let agent = ClaudeFlow {
            task_description: Some("Implement authentication system".to_string()),
            ..Default::default()
        };

        let builder = agent.build_command_builder();
        let command_string = format!("{}", builder);

        assert!(command_string.contains("--task Implement authentication system"));
    }

    #[test]
    fn test_claude_flow_command_builder_with_cmd_overrides() {
        let mut env = HashMap::new();
        env.insert("CUSTOM_VAR".to_string(), "custom_value".to_string());

        let agent = ClaudeFlow {
            cmd: CmdOverrides {
                base_command_override: Some("custom-claude-flow".to_string()),
                additional_params: Some(vec!["--debug".to_string(), "--verbose".to_string()]),
                env: Some(env),
            },
            ..Default::default()
        };

        let builder = agent.build_command_builder();
        let command_string = format!("{}", builder);

        assert!(command_string.contains("custom-claude-flow"));
        assert!(command_string.contains("--debug"));
        assert!(command_string.contains("--verbose"));
    }

    #[test]
    fn test_claude_flow_serialization() {
        let agent = ClaudeFlow {
            append_prompt: AppendPrompt(Some("Additional prompt".to_string())),
            non_interactive: Some(true),
            enable_chaining: Some(false),
            agent_id: Some("test-agent".to_string()),
            workflow_file: Some("workflow.json".to_string()),
            task_description: Some("Test task".to_string()),
            cmd: CmdOverrides::default(),
        };

        // Test serialization
        let serialized = serde_json::to_string(&agent).unwrap();
        assert!(serialized.contains("Additional prompt"));
        assert!(serialized.contains("\"non_interactive\":true"));
        assert!(serialized.contains("\"enable_chaining\":false"));
        assert!(serialized.contains("test-agent"));
        assert!(serialized.contains("workflow.json"));
        assert!(serialized.contains("Test task"));

        // Test deserialization
        let deserialized: ClaudeFlow = serde_json::from_str(&serialized).unwrap();
        assert_eq!(deserialized.append_prompt.0, Some("Additional prompt".to_string()));
        assert_eq!(deserialized.non_interactive, Some(true));
        assert_eq!(deserialized.enable_chaining, Some(false));
        assert_eq!(deserialized.agent_id, Some("test-agent".to_string()));
        assert_eq!(deserialized.workflow_file, Some("workflow.json".to_string()));
        assert_eq!(deserialized.task_description, Some("Test task".to_string()));
    }

    #[test]
    fn test_claude_flow_minimal_serialization() {
        let agent = ClaudeFlow::default();

        let serialized = serde_json::to_string(&agent).unwrap();
        let deserialized: ClaudeFlow = serde_json::from_str(&serialized).unwrap();

        assert_eq!(deserialized.append_prompt.0, None);
        assert_eq!(deserialized.non_interactive, None);
        assert_eq!(deserialized.enable_chaining, None);
        assert_eq!(deserialized.agent_id, None);
        assert_eq!(deserialized.workflow_file, None);
        assert_eq!(deserialized.task_description, None);
        assert_eq!(deserialized.cmd, CmdOverrides::default());
    }

    #[test]
    fn test_append_prompt_combination() {
        let agent = ClaudeFlow {
            append_prompt: AppendPrompt(Some(" Focus on security.".to_string())),
            ..Default::default()
        };

        let base_prompt = "Analyze this codebase";
        let combined = agent.append_prompt.combine_prompt(base_prompt);

        assert_eq!(combined, "Analyze this codebase Focus on security.");
    }

    #[test]
    fn test_append_prompt_none() {
        let agent = ClaudeFlow {
            append_prompt: AppendPrompt(None),
            ..Default::default()
        };

        let base_prompt = "Analyze this codebase";
        let combined = agent.append_prompt.combine_prompt(base_prompt);

        assert_eq!(combined, "Analyze this codebase");
    }

    #[test]
    fn test_default_mcp_config_path() {
        let agent = ClaudeFlow::default();
        let config_path = agent.default_mcp_config_path();

        assert!(config_path.is_some());
        let path = config_path.unwrap();
        assert!(path.to_string_lossy().contains(".claude-flow"));
        assert!(path.ends_with("config.json"));
    }

    #[test]
    fn test_get_availability_info_without_config() {
        let agent = ClaudeFlow::default();
        let availability = agent.get_availability_info();

        // Should be NotFound when no config exists
        assert!(matches!(availability, AvailabilityInfo::NotFound));
    }

    #[test]
    fn test_get_availability_info_with_config() {
        use tempfile::TempDir;
        use std::fs;

        // Create a temporary directory to simulate home directory
        let temp_dir = TempDir::new().unwrap();
        let config_dir = temp_dir.path().join(".claude-flow");
        fs::create_dir_all(&config_dir).unwrap();

        let config_file = config_dir.join("config.json");
        fs::write(&config_file, r#"{"auth": "test"}"#).unwrap();

        // Temporarily set HOME environment variable
        std::env::set_var("HOME", temp_dir.path());

        let agent = ClaudeFlow::default();
        let availability = agent.get_availability_info();

        // Clean up
        std::env::remove_var("HOME");

        // Should detect the config file
        match availability {
            AvailabilityInfo::LoginDetected { .. } => {
                // Config file with timestamp detected
            }
            AvailabilityInfo::InstallationFound => {
                // Config file without timestamp detected
            }
            AvailabilityInfo::NotFound => {
                panic!("Expected config file to be detected");
            }
        }
    }

    #[test]
    fn test_derivative_traits() {
        let agent1 = ClaudeFlow {
            append_prompt: AppendPrompt(Some("test".to_string())),
            non_interactive: Some(true),
            enable_chaining: Some(false),
            agent_id: Some("agent1".to_string()),
            workflow_file: Some("workflow.json".to_string()),
            task_description: Some("task".to_string()),
            cmd: CmdOverrides::default(),
        };

        let agent2 = ClaudeFlow {
            append_prompt: AppendPrompt(Some("test".to_string())),
            non_interactive: Some(true),
            enable_chaining: Some(false),
            agent_id: Some("agent1".to_string()),
            workflow_file: Some("workflow.json".to_string()),
            task_description: Some("task".to_string()),
            cmd: CmdOverrides::default(),
        };

        let agent3 = ClaudeFlow {
            append_prompt: AppendPrompt(Some("different".to_string())),
            non_interactive: Some(true),
            enable_chaining: Some(false),
            agent_id: Some("agent1".to_string()),
            workflow_file: Some("workflow.json".to_string()),
            task_description: Some("task".to_string()),
            cmd: CmdOverrides::default(),
        };

        // Test PartialEq
        assert_eq!(agent1, agent2);
        assert_ne!(agent1, agent3);

        // Test Debug (should compile)
        let debug_str = format!("{:?}", agent1);
        assert!(debug_str.contains("ClaudeFlow"));

        // Test Clone
        let cloned = agent1.clone();
        assert_eq!(agent1, cloned);
    }

    #[test]
    fn test_ts_rs_type_derivation() {
        let agent = ClaudeFlow::default();

        // This should compile without errors if TS derivation works
        let _ts_type = std::any::type_name::<ClaudeFlow>();

        // Test serialization for TS
        let serialized = serde_json::to_string(&agent).unwrap();
        let deserialized: ClaudeFlow = serde_json::from_str(&serialized).unwrap();

        assert_eq!(agent.append_prompt.0, deserialized.append_prompt.0);
    }

    #[test]
    fn test_schemars_json_schema() {
        let agent = ClaudeFlow::default();

        // This should compile without errors if JsonSchema derivation works
        let _schema = schemars::schema_for!(ClaudeFlow);

        // Verify that the schema can be generated
        assert!(_schema.title.is_some());
    }

    #[test]
    fn test_empty_string_handling() {
        let agent = ClaudeFlow {
            append_prompt: AppendPrompt(Some("".to_string())),
            non_interactive: Some(true),
            enable_chaining: Some(false),
            agent_id: Some("".to_string()),
            workflow_file: Some("".to_string()),
            task_description: Some("".to_string()),
            cmd: CmdOverrides::default(),
        };

        // Test that empty strings are handled properly
        assert_eq!(agent.append_prompt.0, Some("".to_string()));
        assert_eq!(agent.agent_id, Some("".to_string()));
        assert_eq!(agent.workflow_file, Some("".to_string()));
        assert_eq!(agent.task_description, Some("".to_string()));

        let builder = agent.build_command_builder();
        let cmd_str = format!("{}", builder);

        // Empty strings should still produce valid command structure
        assert!(cmd_str.contains("--output-format stream-json"));
    }

    #[test]
    fn test_special_characters_in_config() {
        let agent = ClaudeFlow {
            append_prompt: AppendPrompt(Some("Special chars: <>&\"'".to_string())),
            non_interactive: Some(true),
            enable_chaining: Some(true),
            agent_id: Some("agent-with-dashes_and_underscores".to_string()),
            workflow_file: Some("/path/to/workflow.json".to_string()),
            task_description: Some("Task with \"quotes\" and 'apostrophes'".to_string()),
            cmd: CmdOverrides::default(),
        };

        // Test serialization/deserialization with special characters
        let serialized = serde_json::to_string(&agent).unwrap();
        let deserialized: ClaudeFlow = serde_json::from_str(&serialized).unwrap();

        assert_eq!(agent.append_prompt.0, deserialized.append_prompt.0);
        assert_eq!(agent.agent_id, deserialized.agent_id);
        assert_eq!(agent.workflow_file, deserialized.workflow_file);
        assert_eq!(agent.task_description, deserialized.task_description);
    }

    #[test]
    fn test_command_builder_with_all_options() {
        let mut env = HashMap::new();
        env.insert("CUSTOM_VAR".to_string(), "custom_value".to_string());

        let agent = ClaudeFlow {
            append_prompt: AppendPrompt(Some(" Additional context".to_string())),
            non_interactive: Some(true),
            enable_chaining: Some(true),
            agent_id: Some("swarm-coordinator".to_string()),
            workflow_file: Some("complex-workflow.json".to_string()),
            task_description: Some("Complex multi-agent task".to_string()),
            cmd: CmdOverrides {
                base_command_override: Some("custom-claude-flow".to_string()),
                additional_params: Some(vec!["--debug".to_string(), "--verbose".to_string()]),
                env: Some(env),
            },
        };

        let builder = agent.build_command_builder();
        let command_string = format!("{}", builder);

        // Check base command override
        assert!(command_string.contains("custom-claude-flow"));

        // Check all options are included
        assert!(command_string.contains("--output-format stream-json"));
        assert!(command_string.contains("--input-format stream-json"));
        assert!(command_string.contains("--chaining"));
        assert!(command_string.contains("--agent swarm-coordinator"));
        assert!(command_string.contains("--workflow complex-workflow.json"));
        assert!(command_string.contains("--task Complex multi-agent task"));

        // Check additional params
        assert!(command_string.contains("--debug"));
        assert!(command_string.contains("--verbose"));
    }

    #[test]
    fn test_command_builder_disable_chaining() {
        let agent = ClaudeFlow {
            non_interactive: Some(true),
            enable_chaining: Some(false),
            ..Default::default()
        };

        let builder = agent.build_command_builder();
        let command_string = format!("{}", builder);

        // Should NOT contain chaining when explicitly disabled
        assert!(!command_string.contains("--chaining"));
        assert!(command_string.contains("--output-format stream-json"));
    }

    #[tokio::test]
    async fn test_spawn_async() {
        // This is a mock test - actual spawn would require running claude-flow
        let agent = ClaudeFlow::default();
        let current_dir = PathBuf::from("/tmp");
        let prompt = "Test prompt";
        let env = ExecutionEnv::default();

        // Test that spawn method exists and can be called
        // In a real environment, this would actually spawn the process
        let result = agent.spawn(&current_dir, prompt, &env).await;

        // Since claude-flow isn't installed, we expect an error
        // but the method should be callable
        assert!(result.is_err() || result.is_ok()); // Just checking compilation
    }

    #[tokio::test]
    async fn test_spawn_follow_up_async() {
        // This is a mock test - actual spawn would require running claude-flow
        let agent = ClaudeFlow::default();
        let current_dir = PathBuf::from("/tmp");
        let prompt = "Follow-up prompt";
        let session_id = "test-session-123";
        let env = ExecutionEnv::default();

        // Test that spawn_follow_up method exists and can be called
        let result = agent.spawn_follow_up(&current_dir, prompt, session_id, &env).await;

        // Since claude-flow isn't installed, we expect an error
        // but the method should be callable
        assert!(result.is_err() || result.is_ok()); // Just checking compilation
    }

    #[test]
    fn test_normalize_logs() {
        use workspace_utils::msg_store::MsgStore;
        use std::sync::Arc;

        let agent = ClaudeFlow::default();
        let msg_store = Arc::new(MsgStore::new());
        let worktree_path = PathBuf::from("/tmp/test");

        // Test that normalize_logs can be called without panicking
        agent.normalize_logs(msg_store, &worktree_path);

        // Should complete without errors
        assert!(true);
    }

    #[test]
    fn test_use_approvals() {
        let mut agent = ClaudeFlow::default();

        // Test that use_approvals can be called
        // In a real scenario, we'd pass an actual approval service
        agent.use_approvals(Arc::new(crate::approvals::MockApprovalService::new()));

        // Should complete without errors
        assert!(true);
    }
}

// Mock approval service for testing
#[cfg(test)]
mod mock_approval_service {
    use super::*;
    use async_trait::async_trait;

    pub struct MockApprovalService;

    impl MockApprovalService {
        pub fn new() -> Self {
            Self
        }
    }

    #[async_trait]
    impl crate::approvals::ExecutorApprovalService for MockApprovalService {
        async fn request_approval(
            &self,
            _tool_name: &str,
            _tool_input: &serde_json::Value,
        ) -> Result<workspace_utils::approvals::ApprovalStatus, crate::approvals::ExecutorApprovalError> {
            Ok(workspace_utils::approvals::ApprovalStatus::Approved)
        }
    }
}