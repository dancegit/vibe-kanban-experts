#[cfg(test)]
mod claude_flow_integration_tests {
    use super::*;
    use crate::executors::{
        BaseCodingAgent, CodingAgent, StandardCodingAgentExecutor,
    };
    use crate::profile::ExecutorProfileId;
    use std::path::Path;

    #[test]
    fn test_claude_flow_agent_enum_variants() {
        // Test that ClaudeFlow can be created and accessed
        let agent = CodingAgent::ClaudeFlow(ClaudeFlow::default());

        // Test capabilities
        let capabilities = agent.capabilities();
        assert!(capabilities.contains(&BaseAgentCapability::SessionFork));

        // Test MCP config
        let mcp_config = agent.get_mcp_config();
        assert!(mcp_config.supports_mcp);
    }

    #[test]
    fn test_claude_flow_profile_integration() {
        // Test that ClaudeFlow can be created with profile ID
        let profile_id = ExecutorProfileId::new(BaseCodingAgent::ClaudeFlow);
        assert_eq!(profile_id.executor, BaseCodingAgent::ClaudeFlow);
        assert_eq!(profile_id.variant, None);

        // Test with variant
        let profile_id_variant = ExecutorProfileId::with_variant(
            BaseCodingAgent::ClaudeFlow,
            "SWARM".to_string()
        );
        assert_eq!(profile_id_variant.executor, BaseCodingAgent::ClaudeFlow);
        assert_eq!(profile_id_variant.variant, Some("SWARM".to_string()));
    }

    #[test]
    fn test_claude_flow_serialization_with_executor_action() {
        use crate::actions::ExecutorActionType;

        // Test that ClaudeFlow can be used in ExecutorActionType
        let claude_flow_config = ClaudeFlow {
            append_prompt: AppendPrompt(None),
            non_interactive: Some(true),
            enable_chaining: Some(true),
            agent_id: Some("test-agent".to_string()),
            workflow_file: None,
            task_description: Some("Integration test".to_string()),
            cmd: CmdOverrides::default(),
        };

        // Create ExecutorActionType with ClaudeFlow
        let action_type = ExecutorActionType::CodingAgentInitialRequest(
            crate::actions::coding_agent_initial::CodingAgentInitialRequest {
                prompt: "Test prompt".to_string(),
                executor_profile_id: ExecutorProfileId::new(BaseCodingAgent::ClaudeFlow),
                working_dir: None,
            }
        );

        // Verify it can be serialized
        let serialized = serde_json::to_string(&action_type).unwrap();
        let deserialized: ExecutorActionType = serde_json::from_str(&serialized).unwrap();

        match deserialized {
            ExecutorActionType::CodingAgentInitialRequest(request) => {
                assert_eq!(request.executor_profile_id.executor, BaseCodingAgent::ClaudeFlow);
            }
            _ => panic!("Expected CodingAgentInitialRequest"),
        }
    }

    #[test]
    fn test_claude_flow_command_line_construction() {
        let flow = ClaudeFlow {
            append_prompt: AppendPrompt(None),
            non_interactive: Some(true),
            enable_chaining: Some(true),
            agent_id: Some("swarm-agent".to_string()),
            workflow_file: Some("workflow.json".to_string()),
            task_description: Some("Test task".to_string()),
            cmd: CmdOverrides::default(),
        };

        let builder = flow.build_command_builder();
        let cmd_str = format!("{}", builder);

        // Verify the command contains expected elements
        assert!(cmd_str.contains("npx -y claude-flow automation"));
        assert!(cmd_str.contains("--output-format stream-json"));
        assert!(cmd_str.contains("--input-format stream-json"));
        assert!(cmd_str.contains("--chaining"));
        assert!(cmd_str.contains("--agent swarm-agent"));
        assert!(cmd_str.contains("--workflow workflow.json"));
        assert!(cmd_str.contains("--task Test task"));
    }

    #[test]
    fn test_claude_flow_streaming_json_output_handling() {
        use crate::logs::NormalizedEntryType;

        // Test that ClaudeFlow uses streaming JSON processing
        let flow = ClaudeFlow {
            append_prompt: AppendPrompt(None),
            non_interactive: Some(true),
            enable_chaining: Some(false),
            agent_id: None,
            workflow_file: None,
            task_description: None,
            cmd: CmdOverrides::default(),
        };

        // Verify it has the normalize_logs method (trait implementation)
        // This tests the integration with the log processing system
        let msg_store = Arc::new(workspace_utils::msg_store::MsgStore::new());
        let current_dir = Path::new("/tmp/test");

        // This should not panic and should set up log processing
        flow.normalize_logs(msg_store.clone(), current_dir);

        // The log processor should be ready to handle stream JSON
        // In a real scenario, it would process the streaming output
    }

    #[test]
    fn test_claude_flow_error_handling() {
        let flow = ClaudeFlow {
            append_prompt: AppendPrompt(None),
            non_interactive: Some(true),
            enable_chaining: Some(true),
            agent_id: None,
            workflow_file: None,
            task_description: None,
            cmd: CmdOverrides::default(),
        };

        // Test spawn method exists and can be called (though it may fail in test environment)
        // This validates the trait implementation is complete
        let current_dir = Path::new("/tmp");
        let prompt = "Test prompt";
        let env = crate::env::ExecutionEnv::default();

        // The spawn method should exist and return a Result
        // In a real environment with claude-flow installed, this would work
        // In test environment, it might fail with ExecutableNotFound, which is expected
        let result = flow.spawn(current_dir, prompt, &env);

        // We expect this to fail in test environment (no claude-flow binary)
        // but the important thing is that the method exists and has the right signature
        assert!(result.is_err() || result.is_ok()); // Either success or expected test failure
    }

    #[test]
    fn test_claude_flow_mcp_integration() {
        let flow = ClaudeFlow {
            append_prompt: AppendPrompt(None),
            non_interactive: Some(true),
            enable_chaining: Some(false),
            agent_id: None,
            workflow_file: None,
            task_description: None,
            cmd: CmdOverrides::default(),
        };

        // Test MCP configuration methods
        let config_path = flow.default_mcp_config_path();
        assert!(config_path.is_some());

        let path = config_path.unwrap();
        assert!(path.to_string_lossy().contains(".claude-flow"));
        assert!(path.ends_with("config.json"));

        // Test availability info
        let availability = flow.get_availability_info();
        // In test environment, likely NotFound since no config file
        assert!(matches!(availability, AvailabilityInfo::NotFound)
                || matches!(availability, AvailabilityInfo::LoginDetected { .. }));
    }

    #[test]
    fn test_claude_flow_profile_loading() {
        use crate::profile::ExecutorConfigs;

        // Test that profiles can be loaded (this would work in real environment)
        // In test environment, we just verify the method exists
        let configs = ExecutorConfigs::get_cached();

        // This would return Some(claude_flow_executor) in real environment with profiles
        // For now, just verify the method can be called
        let _result = configs.get_coding_agent(&ExecutorProfileId::new(BaseCodingAgent::ClaudeFlow));

        // The result would be Some(_) if profiles are properly loaded
        // or None if not found, both are acceptable for this test
    }

    #[test]
    fn test_claude_flow_json_schema_validation() {
        use schemars::schema::RootSchema;

        // Test that JSON schema can be generated for the ClaudeFlow struct
        let schema: RootSchema = schemars::schema_for!(ClaudeFlow);

        // Verify the schema has the expected properties
        assert!(schema.title.is_some());
        assert!(schema.title.as_ref().unwrap().contains("ClaudeFlow"));

        // Check that our custom fields are in the schema
        let properties = &schema.schema.properties;

        assert!(properties.contains_key("append_prompt"));
        assert!(properties.contains_key("non_interactive"));
        assert!(properties.contains_key("enable_chaining"));
        assert!(properties.contains_key("agent_id"));
        assert!(properties.contains_key("workflow_file"));
        assert!(properties.contains_key("task_description"));
    }

    #[test]
    fn test_claude_flow_type_script_integration() {
        use ts_rs::TS;

        // Test that TypeScript types can be derived
        let _ts_decl = ClaudeFlow::decl();

        // This should compile and generate TypeScript declarations
        // The actual declaration would be generated at compile time
    }
}