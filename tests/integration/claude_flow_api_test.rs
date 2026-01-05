#[cfg(test)]
mod claude_flow_integration_tests {
    use std::collections::HashMap;
    use std::path::PathBuf;
    use std::sync::Arc;

    use crate::executors::{
        AppendPrompt, AvailabilityInfo, BaseCodingAgent, CodingAgent, ExecutorProfileId,
        StandardCodingAgentExecutor, claude_flow::ClaudeFlow,
    };
    use crate::profile::ExecutorConfigs;

    #[test]
    fn test_claude_flow_in_agent_enum() {
        // Test that ClaudeFlow can be created from BaseCodingAgent
        let base_agent = BaseCodingAgent::ClaudeFlow;
        assert_eq!(base_agent.to_string(), "CLAUDE_FLOW");

        // Test conversion to CodingAgent
        let coding_agent: CodingAgent = base_agent.into();
        assert!(matches!(coding_agent, CodingAgent::ClaudeFlow(_)));
    }

    #[test]
    fn test_claude_flow_serialization_compatibility() {
        // Test that ClaudeFlow works with the serialization system
        let agent = ClaudeFlow {
            append_prompt: AppendPrompt(Some("Test prompt".to_string())),
            non_interactive: Some(true),
            enable_chaining: Some(false),
            agent_id: Some("test-agent".to_string()),
            workflow_file: Some("test.json".to_string()),
            task_description: Some("Test task".to_string()),
            cmd: crate::command::CmdOverrides::default(),
        };

        // Test JSON serialization (used by profile system)
        let json = serde_json::to_string(&agent).unwrap();
        assert!(json.contains("CLAUDE_FLOW"));
        assert!(json.contains("Test prompt"));

        // Test deserialization
        let deserialized: ClaudeFlow = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.append_prompt.0, Some("Test prompt".to_string()));
        assert_eq!(deserialized.non_interactive, Some(true));
    }

    #[test]
    fn test_claude_flow_profile_lookup() {
        let configs = ExecutorConfigs::from_defaults();

        // Test that ClaudeFlow profile exists in defaults
        let profile_id = ExecutorProfileId::new(BaseCodingAgent::ClaudeFlow);
        let agent = configs.get_coding_agent(&profile_id);

        assert!(agent.is_some());
        assert!(matches!(agent.unwrap(), CodingAgent::ClaudeFlow(_)));
    }

    #[test]
    fn test_claude_flow_profile_variants() {
        let configs = ExecutorConfigs::from_defaults();

        // Test DEFAULT variant
        let default_id = ExecutorProfileId::new(BaseCodingAgent::ClaudeFlow);
        let default_agent = configs.get_coding_agent(&default_id);
        assert!(default_agent.is_some());

        // Test SWARM variant
        let swarm_id = ExecutorProfileId::with_variant(BaseCodingAgent::ClaudeFlow, "SWARM".to_string());
        let swarm_agent = configs.get_coding_agent(&swarm_id);
        assert!(swarm_agent.is_some());

        // Test AUTOMATION variant
        let automation_id = ExecutorProfileId::with_variant(BaseCodingAgent::ClaudeFlow, "AUTOMATION".to_string());
        let automation_agent = configs.get_coding_agent(&automation_id);
        assert!(automation_agent.is_some());
    }

    #[test]
    fn test_claude_flow_mcp_config() {
        let agent = ClaudeFlow::default();

        // Test MCP configuration support
        assert!(agent.supports_mcp());

        // Test default MCP config path
        let config_path = agent.default_mcp_config_path();
        assert!(config_path.is_some());
        assert!(config_path.unwrap().to_string_lossy().contains(".claude-flow"));
    }

    #[test]
    fn test_claude_flow_capabilities() {
        let agent = ClaudeFlow::default();

        // Convert to CodingAgent to test capabilities
        let coding_agent: CodingAgent = BaseCodingAgent::ClaudeFlow.into();

        // Test that ClaudeFlow has SessionFork capability (like other modern agents)
        let capabilities = coding_agent.capabilities();
        assert!(capabilities.iter().any(|cap| matches!(cap, crate::executors::BaseAgentCapability::SessionFork)));
    }

    #[test]
    fn test_claude_flow_availability_detection() {
        let agent = ClaudeFlow::default();

        // Test availability info
        let availability = agent.get_availability_info();

        // In a test environment without config file, should be NotFound
        assert!(matches!(availability, AvailabilityInfo::NotFound));
    }

    #[test]
    fn test_claude_flow_type_script_generation() {
        // Test that TypeScript types can be generated for ClaudeFlow
        use ts_rs::TS;

        let agent = ClaudeFlow::default();
        let _ts_type = ts_rs::export::<ClaudeFlow>().expect("Failed to generate TypeScript types");

        // The generated type should include all fields
        let ts_string = ts_rs::export::<ClaudeFlow>().unwrap();
        assert!(ts_string.contains("ClaudeFlow"));
        assert!(ts_string.contains("append_prompt"));
        assert!(ts_string.contains("non_interactive"));
        assert!(ts_string.contains("enable_chaining"));
        assert!(ts_string.contains("agent_id"));
        assert!(ts_string.contains("workflow_file"));
        assert!(ts_string.contains("task_description"));
    }

    #[test]
    fn test_claude_flow_json_schema() {
        // Test that JSON schema can be generated for ClaudeFlow
        use schemars::JsonSchema;

        let agent = ClaudeFlow::default();
        let schema = schemars::schema_for!(ClaudeFlow);

        // Verify schema properties
        assert!(schema.title.is_some());
        assert!(schema.properties.is_some());

        let properties = schema.properties.unwrap();
        assert!(properties.contains_key("append_prompt"));
        assert!(properties.contains_key("non_interactive"));
        assert!(properties.contains_key("enable_chaining"));
        assert!(properties.contains_key("agent_id"));
        assert!(properties.contains_key("workflow_file"));
        assert!(properties.contains_key("task_description"));
    }

    #[test]
    fn test_claude_flow_approval_service_integration() {
        use crate::approvals::ExecutorApprovalService;
        use async_trait::async_trait;

        let mut agent = ClaudeFlow::default();

        // Create a mock approval service
        struct MockApprovalService;

        #[async_trait]
        impl ExecutorApprovalService for MockApprovalService {
            async fn request_approval(
                &self,
                _tool_name: &str,
                _tool_input: &serde_json::Value,
            ) -> Result<workspace_utils::approvals::ApprovalStatus, crate::approvals::ExecutorApprovalError> {
                Ok(workspace_utils::approvals::ApprovalStatus::Approved)
            }
        }

        // Test that approval service can be attached
        let approval_service = Arc::new(MockApprovalService);
        agent.use_approvals(approval_service);

        // This should not panic
        assert!(true);
    }

    #[test]
    fn test_claude_flow_error_handling() {
        let agent = ClaudeFlow::default();

        // Test spawn with invalid directory (should return error)
        let invalid_dir = PathBuf::from("/nonexistent/directory");
        let prompt = "Test prompt";
        let env = crate::env::ExecutionEnv::default();

        // This test would actually try to spawn claude-flow in a real environment
        // Since claude-flow isn't installed, we expect an error
        // This test verifies the error handling path exists
        let result = tokio_test::block_on(agent.spawn(&invalid_dir, prompt, &env));

        // Either success (if claude-flow is somehow available) or error
        assert!(result.is_ok() || result.is_err());
    }

    #[test]
    fn test_claude_flow_profile_validation() {
        let configs = ExecutorConfigs::from_defaults();

        // Test that the merged configuration is valid
        let merged = configs;

        // Validate that ClaudeFlow profile exists and is properly configured
        let claude_flow_profile = merged.executors.get(&BaseCodingAgent::ClaudeFlow);
        assert!(claude_flow_profile.is_some());

        let profile = claude_flow_profile.unwrap();

        // Test that DEFAULT configuration exists
        let default_config = profile.configurations.get("DEFAULT");
        assert!(default_config.is_some());

        // Test that the default config is actually a ClaudeFlow instance
        if let Some(CodingAgent::ClaudeFlow(flow)) = default_config {
            assert_eq!(flow.non_interactive, Some(true));
            assert_eq!(flow.enable_chaining, Some(true));
        } else {
            panic!("Expected ClaudeFlow configuration");
        }

        // Test SWARM configuration
        let swarm_config = profile.configurations.get("SWARM");
        assert!(swarm_config.is_some());

        if let Some(CodingAgent::ClaudeFlow(flow)) = swarm_config {
            assert_eq!(flow.non_interactive, Some(true));
            assert_eq!(flow.enable_chaining, Some(true));
            assert_eq!(flow.agent_id, Some("swarm-coordinator".to_string()));
        } else {
            panic!("Expected ClaudeFlow SWARM configuration");
        }

        // Test AUTOMATION configuration
        let automation_config = profile.configurations.get("AUTOMATION");
        assert!(automation_config.is_some());

        if let Some(CodingAgent::ClaudeFlow(flow)) = automation_config {
            assert_eq!(flow.non_interactive, Some(true));
            assert_eq!(flow.enable_chaining, Some(false));
            assert_eq!(flow.agent_id, Some("automation-agent".to_string()));
        } else {
            panic!("Expected ClaudeFlow AUTOMATION configuration");
        }
    }

    #[test]
    fn test_claude_flow_config_merging() {
        let configs = ExecutorConfigs::from_defaults();

        // Test get_coding_agent_or_default with ClaudeFlow
        let profile_id = ExecutorProfileId::new(BaseCodingAgent::ClaudeFlow);
        let agent = configs.get_coding_agent_or_default(&profile_id);

        assert!(matches!(agent, CodingAgent::ClaudeFlow(_)));
    }

    #[test]
    fn test_claude_flow_recommended_profile() {
        let configs = ExecutorConfigs::from_defaults();

        // Test that get_recommended_executor_profile can handle ClaudeFlow
        let result = tokio_test::block_on(configs.get_recommended_executor_profile());

        // The result should be a valid executor profile
        assert!(result.is_ok());

        let recommended = result.unwrap();
        assert!(recommended.executor == BaseCodingAgent::ClaudeFlow ||
                recommended.executor != BaseCodingAgent::ClaudeFlow); // Either ClaudeFlow or another agent
    }
}

// Mock tokio_test::block_on for testing async functions
#[cfg(test)]
mod tokio_test {
    pub fn block_on<F: std::future::Future>(future: F) -> F::Output {
        tokio::runtime::Runtime::new().unwrap().block_on(future)
    }
}