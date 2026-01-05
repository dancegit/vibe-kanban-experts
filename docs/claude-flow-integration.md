# Claude-Flow Agent Integration

This document describes the integration of claude-flow as a first-class agent in the Vibe Kanban system.

## Overview

Claude-Flow is now supported as a coding agent option, enabling users to leverage swarm-based AI coordination and automation capabilities directly within the Vibe Kanban interface.

## Features

### Core Capabilities
- **Non-interactive mode execution** for automation
- **JSON streaming output** for real-time processing
- **Agent chaining** for complex workflows
- **Swarm coordination** support
- **Workflow file support** for complex configurations
- **Task-based automation** commands

### Configuration Options

The claude-flow agent supports the following configuration parameters:

```typescript
interface ClaudeFlow {
  append_prompt?: string;           // Additional context to append to prompts
  non_interactive?: boolean;        // Run in non-interactive mode
  enable_chaining?: boolean;        // Enable stream chaining between agents
  agent_id?: string;                // Specific agent to run
  workflow_file?: string;           // Path to workflow configuration file
  task_description?: string;        // Task description for automation
}
```

### Profile Variants

Three pre-configured profile variants are available:

1. **DEFAULT**: Basic claude-flow with non-interactive mode and chaining enabled
2. **SWARM**: Optimized for swarm coordination with "swarm-coordinator" agent
3. **AUTOMATION**: Focused on single-agent automation tasks

## Implementation Details

### Agent Integration

The claude-flow agent is implemented following the existing agent pattern:

- **Enum Integration**: Added to `CodingAgent` enum with proper serialization
- **Executor Implementation**: Implements `StandardCodingAgentExecutor` trait
- **Log Processing**: Uses Claude's JSON streaming processor for output normalization
- **Profile System**: Integrated with existing profile configuration system

### Command Construction

The agent constructs commands based on the configuration:

```rust
// Base command selection
let base_cmd = if self.non_interactive.unwrap_or(false) {
    "npx -y claude-flow automation"
} else {
    "npx -y claude-flow"
};

// Stream JSON configuration
builder = builder
    .params(["--output-format", "stream-json"])
    .extend_params(["--input-format", "stream-json"]);

// Feature flags
if self.enable_chaining.unwrap_or(false) {
    builder = builder.extend_params(["--chaining"]);
}
```

### Streaming Output Processing

Claude-flow outputs JSON in a format compatible with Claude's log processor:

- **Stream Processing**: Uses `ClaudeLogProcessor` with `HistoryStrategy::Default`
- **Output Normalization**: Converts claude-flow messages to normalized entries
- **Error Handling**: Processes stderr through standard processors

## API Integration

### Endpoint Compatibility

Claude-flow integrates seamlessly with existing API endpoints:

- **Session Management**: Works with `/api/sessions/*` endpoints
- **Task Execution**: Compatible with task attempt creation
- **Process Monitoring**: Integrated with execution process tracking
- **WebSocket Support**: Real-time output streaming through existing WS infrastructure

### TypeScript Types

TypeScript types are automatically generated through the `ts-rs` derive macros:

```typescript
export type ClaudeFlow = {
  append_prompt?: string;
  non_interactive?: boolean;
  enable_chaining?: boolean;
  agent_id?: string;
  workflow_file?: string;
  task_description?: string;
};
```

## Usage Examples

### Basic Usage

```typescript
// Create a basic claude-flow agent execution
const action = {
  type: "CodingAgentInitialRequest",
  prompt: "Analyze the codebase and suggest improvements",
  executor_profile_id: {
    executor: "CLAUDE_FLOW",
    variant: "DEFAULT"
  }
};
```

### Swarm Coordination

```typescript
// Use swarm-specific configuration
const swarmAction = {
  type: "CodingAgentInitialRequest",
  prompt: "Coordinate a multi-agent analysis of this project",
  executor_profile_id: {
    executor: "CLAUDE_FLOW",
    variant: "SWARM"
  }
};
```

### Automation Mode

```typescript
// Use automation-specific configuration
const automationAction = {
  type: "CodingAgentInitialRequest",
  prompt: "Run automated code quality checks",
  executor_profile_id: {
    executor: "CLAUDE_FLOW",
    variant: "AUTOMATION"
  }
};
```

### Custom Configuration

```typescript
// Use custom configuration
const customConfig = {
  type: "CodingAgentInitialRequest",
  prompt: "Complex task requiring chaining",
  executor_profile_id: {
    executor: "CLAUDE_FLOW",
    variant: "DEFAULT"
  },
  working_dir: "src/"
};

// Override with custom parameters
const customFlow = {
  append_prompt: " Focus on performance optimizations",
  non_interactive: true,
  enable_chaining: true,
  agent_id: "performance-analyzer",
  workflow_file: "performance-workflow.json"
};
```

## Error Handling

### Common Issues

1. **Executable Not Found**: Ensure claude-flow is installed globally via npm
2. **Authentication**: Configure claude-flow authentication (similar to Claude Code)
3. **Workflow Files**: Ensure workflow files exist and are valid JSON
4. **Agent Dependencies**: Verify required agents are available

### Error Recovery

- **Graceful Degradation**: Falls back to basic execution if advanced features unavailable
- **Stream Processing**: Robust error handling in JSON stream parsing
- **Process Management**: Proper cleanup on interruption or failure

## Configuration Files

### Profile Configuration

Claude-flow profiles are defined in `default_profiles.json`:

```json
{
  "CLAUDE_FLOW": {
    "DEFAULT": {
      "CLAUDE_FLOW": {
        "non_interactive": true,
        "enable_chaining": true
      }
    },
    "SWARM": {
      "CLAUDE_FLOW": {
        "non_interactive": true,
        "enable_chaining": true,
        "agent_id": "swarm-coordinator"
      }
    },
    "AUTOMATION": {
      "CLAUDE_FLOW": {
        "non_interactive": true,
        "enable_chaining": false,
        "agent_id": "automation-agent"
      }
    }
  }
}
```

### MCP Configuration

Claude-flow looks for configuration in:
- `$HOME/.claude-flow/config.json`
- Format compatible with existing MCP configuration system

## Testing

### Unit Tests

Comprehensive unit tests cover:
- Configuration serialization/deserialization
- Command line construction
- Error handling scenarios
- Profile integration

### Integration Tests

Integration tests verify:
- Agent selection in UI
- API endpoint compatibility
- Streaming output processing
- Error recovery scenarios

### Test Commands

```bash
# Run unit tests
cargo test claude_flow

# Run integration tests
cargo test claude_flow_integration

# Run all executor tests
cargo test --package executors
```

## Performance Considerations

### Stream Processing
- **Memory Efficient**: Processes JSON streams incrementally
- **Low Latency**: Real-time output streaming to UI
- **Robust Parsing**: Handles partial messages and stream interruptions

### Process Management
- **Graceful Shutdown**: Proper cleanup on process termination
- **Resource Management**: Memory and CPU usage monitoring
- **Concurrent Execution**: Supports multiple simultaneous claude-flow processes

## Security Considerations

### Authentication
- Uses same authentication mechanism as Claude Code
- Secure credential storage in user configuration
- API key management through environment variables

### Sandbox Integration
- Runs within existing containerized environment
- File system access controlled through workspace permissions
- Network access subject to existing security policies

## Future Enhancements

### Planned Features
- **Advanced Workflow Support**: Complex multi-agent orchestration
- **Custom Agent Definitions**: User-defined agent configurations
- **Performance Metrics**: Execution time and resource usage tracking
- **Enhanced Debugging**: Improved error reporting and troubleshooting

### Extensibility
- Plugin system for custom claude-flow extensions
- Custom stream processors for specialized output formats
- Integration with external workflow orchestration systems

## Troubleshooting

### Common Issues and Solutions

1. **Agent Not Available**
   - Verify claude-flow installation: `npm list -g claude-flow`
   - Check PATH configuration
   - Ensure proper npm permissions

2. **Streaming Output Issues**
   - Verify `--output-format stream-json` is set
   - Check for network connectivity issues
   - Review stderr for parsing errors

3. **Workflow File Problems**
   - Validate JSON syntax
   - Check file permissions
   - Verify agent references exist

4. **Performance Issues**
   - Monitor system resources
   - Check for process leaks
   - Review stream processing latency

### Debug Commands

```bash
# Check claude-flow installation
npx claude-flow --version

# Test stream output manually
echo "test prompt" | npx claude-flow --output-format stream-json

# Validate workflow files
cat workflow.json | jq .
```

## Migration Guide

### From Other Agents

Claude-flow can be used as a drop-in replacement for other agents in most scenarios:

1. **Replace executor type** from `CLAUDE_CODE` to `CLAUDE_FLOW`
2. **Adjust configuration** based on workflow requirements
3. **Update prompts** if using swarm-specific features
4. **Test thoroughly** in development environment

### Backward Compatibility

- Maintains full compatibility with existing API endpoints
- Supports all existing workflow patterns
- No breaking changes to frontend interfaces
- Graceful fallback for unavailable features

## Conclusion

The claude-flow integration provides a powerful new option for users requiring advanced AI coordination and automation capabilities. The implementation follows established patterns while adding specialized features for swarm-based AI coordination.

For additional support or feature requests, please refer to the project repository or contact the development team.