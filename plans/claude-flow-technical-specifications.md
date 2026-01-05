# Claude-Flow Integration Technical Specifications

## Interface Definitions

### 1. ClaudeFlow Executor Interface

#### Core Struct Definition

```rust
#[derive(Debug, Clone, Serialize, Deserialize, TS, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct ClaudeFlow {
    /// Extra text appended to the prompt
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub append_prompt: Option<String>,

    /// Output format for streaming (default: "stream-json")
    #[serde(default = "default_output_format")]
    pub output_format: String,

    /// Enable non-interactive mode
    #[serde(default = "default_non_interactive")]
    pub non_interactive: bool,

    /// Enable agent chaining
    #[serde(default = "default_chaining")]
    pub chaining: bool,

    /// Custom task prompt
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub task_prompt: Option<String>,

    /// Workflow configuration JSON
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub workflow_config: Option<WorkflowConfig>,

    /// Environment variables
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub env_vars: Option<HashMap<String, String>>,

    /// Command overrides
    #[serde(flatten)]
    pub cmd: CmdOverrides,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct WorkflowConfig {
    /// Enable automatic chaining
    #[serde(default)]
    pub enable_chaining: bool,

    /// Stream format configuration
    #[serde(default)]
    pub output_format: StreamFormat,

    /// Agent configuration
    #[serde(default)]
    pub agent_config: AgentConfig,

    /// Workflow timeout in seconds
    #[serde(default)]
    pub timeout_seconds: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct StreamFormat {
    /// Message type format
    #[serde(default)]
    pub message_type: MessageType,

    /// Include timestamps
    #[serde(default)]
    pub include_timestamps: bool,

    /// Include metadata
    #[serde(default)]
    pub include_metadata: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct AgentConfig {
    /// Default model to use
    #[serde(default)]
    pub default_model: String,

    /// Maximum tokens per response
    #[serde(default)]
    pub max_tokens: u32,

    /// Temperature for responses
    #[serde(default)]
    pub temperature: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS, JsonSchema)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum MessageType {
    Init,
    Message,
    ToolUse,
    ToolResult,
    Result,
    Error,
}
```

#### Executor Trait Implementation

```rust
#[async_trait]
impl StandardCodingAgentExecutor for ClaudeFlow {
    type Output = SpawnedChild;

    async fn spawn(
        &self,
        env: &ExecutionEnv,
        action: &ExecutorAction,
        approvals_service: Arc<dyn ExecutorApprovalService>,
        msg_store: MsgStore,
    ) -> Result<Self::Output, ExecutorError> {
        // Build command
        let command = self.build_command(action, env)?;

        // Create process
        let mut command = command.build();

        // Configure process
        command
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .stdin(Stdio::piped());

        // Set environment variables
        if let Some(env_vars) = &self.env_vars {
            for (key, value) in env_vars {
                command.env(key, value);
            }
        }

        // Spawn process
        let mut child = command
            .group_spawn()
            .map_err(|e| ExecutorError::SpawnError(e.into()))?;

        // Get stdout for streaming
        let stdout = child
            .inner()
            .stdout
            .take()
            .ok_or_else(|| ExecutorError::Io(std::io::Error::new(
                std::io::ErrorKind::Other,
                "Failed to get stdout",
            )))?;

        // Start stream processing
        let stream_processor = ClaudeFlowStreamProcessor::new(
            stdout,
            msg_store.clone(),
            self.output_format.clone(),
        );

        tokio::spawn(async move {
            if let Err(e) = stream_processor.process_stream().await {
                tracing::error!("Stream processing error: {}", e);
            }
        });

        Ok(SpawnedChild::new(child))
    }

    fn get_capabilities(&self) -> &[BaseAgentCapability] {
        &[
            BaseAgentCapability::SessionFork,
            BaseAgentCapability::SetupHelper,
        ]
    }

    fn supports_follow_up(&self) -> bool {
        true
    }
}
```

### 2. JSON Stream Parser Interface

```rust
pub struct ClaudeFlowStreamProcessor {
    reader: BufReader<Box<dyn AsyncRead + Send + Unpin>>,
    msg_store: MsgStore,
    output_format: String,
    buffer: String,
    message_count: u64,
    start_time: Instant,
}

impl ClaudeFlowStreamProcessor {
    pub fn new(
        stdout: impl AsyncRead + Send + Unpin,
        msg_store: MsgStore,
        output_format: String,
    ) -> Self {
        Self {
            reader: BufReader::new(Box::new(stdout)),
            msg_store,
            output_format,
            buffer: String::new(),
            message_count: 0,
            start_time: Instant::now(),
        }
    }

    pub async fn process_stream(&mut self) -> Result<(), ExecutorError> {
        let mut line = String::new();

        while let Ok(bytes_read) = self.reader.read_line(&mut line).await {
            if bytes_read == 0 {
                break; // EOF
            }

            let trimmed = line.trim();
            if trimmed.is_empty() {
                line.clear();
                continue;
            }

            match self.parse_message(trimmed).await {
                Ok(message) => {
                    self.message_count += 1;
                    self.handle_message(message).await?;
                }
                Err(e) => {
                    tracing::warn!("Failed to parse message: {}", e);
                    // Continue processing other messages
                }
            }

            line.clear();
        }

        tracing::info!(
            "Stream processing completed. Processed {} messages in {:?}",
            self.message_count,
            self.start_time.elapsed()
        );

        Ok(())
    }

    async fn parse_message(&self, json_str: &str) -> Result<StreamMessage, serde_json::Error> {
        serde_json::from_str::<StreamMessage>(json_str)
    }

    async fn handle_message(&mut self, message: StreamMessage) -> Result<(), ExecutorError> {
        match message {
            StreamMessage::Init { session_id, timestamp } => {
                self.handle_init(session_id, timestamp).await?;
            }
            StreamMessage::Message { role, content } => {
                self.handle_message_content(role, content).await?;
            }
            StreamMessage::ToolUse { id, name, input } => {
                self.handle_tool_use(id, name, input).await?;
            }
            StreamMessage::ToolResult { id, result } => {
                self.handle_tool_result(id, result).await?;
            }
            StreamMessage::Result { status, summary } => {
                self.handle_result(status, summary).await?;
            }
            StreamMessage::Error { code, message } => {
                self.handle_error(code, message).await?;
            }
        }

        Ok(())
    }

    async fn handle_init(&self, session_id: String, timestamp: u64) -> Result<(), ExecutorError> {
        let log_entry = LogEntry::new(
            "claude_flow",
            "init",
            format!("Session initialized: {}", session_id),
            timestamp,
        );

        self.msg_store.add(log_entry).await;
        Ok(())
    }

    async fn handle_message_content(
        &self,
        role: String,
        content: String,
    ) -> Result<(), ExecutorError> {
        let log_entry = LogEntry::new(
            "claude_flow",
            "message",
            format!("[{}]: {}", role, content),
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs(),
        );

        self.msg_store.add(log_entry).await;
        Ok(())
    }

    // Additional handler methods...
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum StreamMessage {
    Init {
        session_id: String,
        timestamp: u64,
    },
    Message {
        role: String,
        content: String,
    },
    ToolUse {
        id: String,
        name: String,
        input: serde_json::Value,
    },
    ToolResult {
        id: String,
        result: serde_json::Value,
    },
    Result {
        status: String,
        summary: String,
    },
    Error {
        code: String,
        message: String,
    },
}
```

### 3. Command Builder Interface

```rust
impl ClaudeFlow {
    fn build_command(
        &self,
        action: &ExecutorAction,
        env: &ExecutionEnv,
    ) -> Result<CommandBuilder, ExecutorError> {
        let mut builder = CommandBuilder::new("npx")
            .params(["-y", "@ruvnet/claude-flow"])
            .params(["--output-format", &self.output_format]);

        // Enable non-interactive mode
        if self.non_interactive {
            builder = builder.params(["--non-interactive"]);
        }

        // Enable chaining if configured
        if self.chaining {
            builder = builder.params(["--chaining"]);
        }

        // Add workflow configuration if present
        if let Some(workflow_config) = &self.workflow_config {
            let config_json = serde_json::to_string(workflow_config)
                .map_err(|e| ExecutorError::Json(e))?;
            builder = builder.params(["--config", &config_json]);
        }

        // Handle action-specific configuration
        match action {
            ExecutorAction::CodingAgentInitial(request) => {
                builder = builder.params(["--task", &request.prompt]);

                // Add working directory
                if let Some(working_dir) = &env.working_dir {
                    builder = builder.params(["--working-dir", working_dir]);
                }
            }
            ExecutorAction::CodingAgentFollowUp(request) => {
                builder = builder.params(["--continue"]);
                builder = builder.params(["--context", &request.message]);
            }
        }

        // Apply command overrides
        if let Some(base_override) = &self.cmd.base_command_override {
            tracing::warn!("Overriding base command with: {}", base_override);
        }

        Ok(builder)
    }
}
```

### 4. Database Schema Extensions

```sql
-- Migration: Add Claude-Flow configuration support
ALTER TABLE executor_profiles
ADD COLUMN claude_flow_config TEXT;

-- Create claude_flow_configs table
CREATE TABLE claude_flow_configs (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    task_attempt_id UUID NOT NULL,
    output_format TEXT NOT NULL DEFAULT 'stream-json',
    non_interactive BOOLEAN NOT NULL DEFAULT true,
    chaining BOOLEAN NOT NULL DEFAULT true,
    task_prompt TEXT,
    workflow_config TEXT,
    env_vars TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    FOREIGN KEY (task_attempt_id) REFERENCES task_attempts (id)
);

-- Create claude_flow_workflows table
CREATE TABLE claude_flow_workflows (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    task_attempt_id UUID NOT NULL,
    workflow_definition TEXT NOT NULL,
    agent_assignments TEXT,
    status TEXT NOT NULL DEFAULT 'pending',
    started_at TIMESTAMPTZ,
    completed_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    FOREIGN KEY (task_attempt_id) REFERENCES task_attempts (id)
);

-- Create stream_messages table
CREATE TABLE stream_messages (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    execution_process_id UUID NOT NULL,
    message_type TEXT NOT NULL,
    message_data TEXT NOT NULL,
    raw_content TEXT NOT NULL,
    sequence_number INTEGER NOT NULL,
    timestamp TIMESTAMPTZ NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    FOREIGN KEY (execution_process_id) REFERENCES execution_processes (id)
);

-- Create agent_states table
CREATE TABLE agent_states (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    execution_process_id UUID NOT NULL,
    agent_id TEXT NOT NULL,
    state_data TEXT NOT NULL,
    status TEXT NOT NULL DEFAULT 'active',
    last_heartbeat TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    FOREIGN KEY (execution_process_id) REFERENCES execution_processes (id)
);

-- Indexes for performance
CREATE INDEX idx_claude_flow_configs_task_attempt ON claude_flow_configs (task_attempt_id);
CREATE INDEX idx_stream_messages_execution_process ON stream_messages (execution_process_id);
CREATE INDEX idx_stream_messages_sequence ON stream_messages (execution_process_id, sequence_number);
CREATE INDEX idx_agent_states_execution_process ON agent_states (execution_process_id);
CREATE INDEX idx_agent_states_status ON agent_states (status);
```

### 5. API Interface Definitions

#### Frontend TypeScript Interfaces

```typescript
interface ClaudeFlowConfig {
  outputFormat: 'stream-json';
  nonInteractive: boolean;
  chaining: boolean;
  taskPrompt?: string;
  workflowConfig?: WorkflowDefinition;
  envVars?: Record<string, string>;
}

interface WorkflowDefinition {
  enableChaining: boolean;
  outputFormat: StreamFormatConfig;
  agentConfig: AgentConfiguration;
  timeoutSeconds: number;
  tasks?: WorkflowTask[];
}

interface WorkflowTask {
  id: string;
  name: string;
  assignTo: string;
  claudePrompt: string;
  depends?: string[];
}

interface StreamMessage {
  type: 'init' | 'message' | 'tool_use' | 'tool_result' | 'result' | 'error';
  timestamp: number;
  data: any;
}

interface ClaudeFlowExecutionState {
  sessionId: string;
  messageCount: number;
  currentStatus: 'initializing' | 'running' | 'completed' | 'error';
  agents: AgentState[];
}

interface AgentState {
  id: string;
  name: string;
  status: 'idle' | 'processing' | 'completed' | 'error';
  lastActivity: number;
}
```

#### API Response Types

```rust
#[derive(Debug, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
pub struct CreateClaudeFlowAttemptResponse {
    pub attempt_id: Uuid,
    pub session_id: Uuid,
    pub status: String,
    pub config: ClaudeFlowConfig,
}

#[derive(Debug, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
pub struct StreamMessageResponse {
    pub message_id: Uuid,
    pub execution_process_id: Uuid,
    pub message_type: String,
    pub content: serde_json::Value,
    pub timestamp: DateTime<Utc>,
    pub sequence_number: u64,
}

#[derive(Debug, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
pub struct ClaudeFlowExecutionStatus {
    pub execution_process_id: Uuid,
    pub status: ExecutionProcessStatus,
    pub message_count: u64,
    pub current_agents: Vec<String>,
    pub progress: ExecutionProgress,
}

#[derive(Debug, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
pub struct ExecutionProgress {
    pub completed_tasks: u32,
    pub total_tasks: u32,
    pub active_agents: u32,
    pub elapsed_time: Duration,
}
```

### 6. Error Handling Interfaces

```rust
#[derive(Debug, Error)]
pub enum ClaudeFlowError {
    #[error("Invalid JSON stream: {0}")]
    InvalidStream(#[from] serde_json::Error),

    #[error("Stream parse error: {message}")]
    StreamParseError { message: String },

    #[error("Process spawn error: {0}")]
    ProcessSpawn(#[from] std::io::Error),

    #[error("Command build error: {0}")]
    CommandBuild(#[from] CommandBuildError),

    #[error("Workflow validation error: {0}")]
    WorkflowValidation(String),

    #[error("Agent chaining error: {0}")]
    AgentChaining(String),

    #[error("Timeout error: operation exceeded {timeout_seconds}s")]
    Timeout { timeout_seconds: u64 },

    #[error("Resource limit exceeded")]
    ResourceLimitExceeded,

    #[error("Authentication required: {0}")]
    AuthRequired(String),

    #[error("Network error: {0}")]
    Network(String),
}

impl ClaudeFlowError {
    pub fn user_friendly_message(&self) -> String {
        match self {
            Self::InvalidStream(_) => "Failed to parse agent response".to_string(),
            Self::StreamParseError { message } => {
                format!("Stream processing error: {}", message)
            }
            Self::ProcessSpawn(_) => "Failed to start Claude-Flow process".to_string(),
            Self::CommandBuild(_) => "Invalid command configuration".to_string(),
            Self::WorkflowValidation(msg) => {
                format!("Invalid workflow configuration: {}", msg)
            }
            Self::AgentChaining(msg) => {
                format!("Agent coordination error: {}", msg)
            }
            Self::Timeout { timeout_seconds } => {
                format!("Operation timed out after {} seconds", timeout_seconds)
            }
            Self::ResourceLimitExceeded => {
                "Resource limit exceeded".to_string()
            }
            Self::AuthRequired(service) => {
                format!("Authentication required for: {}", service)
            }
            Self::Network(msg) => {
                format!("Network error: {}", msg)
            }
        }
    }

    pub fn retryable(&self) -> bool {
        matches!(
            self,
            Self::Network(_) | Self::ProcessSpawn(_) | Self::Timeout { .. }
        )
    }
}
```

### 7. Configuration Management

```rust
#[derive(Debug, Serialize, Deserialize, TS, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct ClaudeFlowSettings {
    /// Default output format
    #[serde(default = "default_output_format")]
    pub default_output_format: String,

    /// Default non-interactive mode
    #[serde(default = "default_non_interactive")]
    pub default_non_interactive: bool,

    /// Default chaining behavior
    #[serde(default = "default_chaining")]
    pub default_chaining: bool,

    /// Default timeout in seconds
    #[serde(default = "default_timeout")]
    pub default_timeout: u64,

    /// Maximum concurrent agents
    #[serde(default = "default_max_agents")]
    pub max_concurrent_agents: u32,

    /// Resource limits
    #[serde(default)]
    pub resource_limits: ResourceLimits,

    /// Logging configuration
    #[serde(default)]
    pub logging: LoggingConfig,
}

#[derive(Debug, Serialize, Deserialize, TS, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct ResourceLimits {
    /// Maximum memory usage in MB
    #[serde(default = "default_max_memory_mb")]
    pub max_memory_mb: u32,

    /// Maximum CPU time in seconds
    #[serde(default = "default_max_cpu_time")]
    pub max_cpu_time: u64,

    /// Maximum file size in MB
    #[serde(default = "default_max_file_size_mb")]
    pub max_file_size_mb: u32,

    /// Maximum network connections
    #[serde(default = "default_max_connections")]
    pub max_connections: u32,
}

#[derive(Debug, Serialize, Deserialize, TS, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct LoggingConfig {
    /// Log level
    #[serde(default = "default_log_level")]
    pub level: String,

    /// Include timestamps
    #[serde(default)]
    pub include_timestamps: bool,

    /// Include metadata
    #[serde(default)]
    pub include_metadata: bool,

    /// Redact sensitive data
    #[serde(default)]
    pub redact_sensitive: bool,
}
```

### 8. Monitoring and Metrics

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionMetrics {
    pub execution_id: Uuid,
    pub start_time: SystemTime,
    pub end_time: Option<SystemTime>,
    pub message_count: u64,
    pub bytes_processed: u64,
    pub agent_count: u32,
    pub memory_peak: u64,
    pub cpu_time: Duration,
    pub network_io: NetworkIO,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkIO {
    pub bytes_sent: u64,
    pub bytes_received: u64,
    pub connections_count: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StreamProcessingStats {
    pub messages_per_second: f64,
    pub average_message_size: f64,
    pub parse_error_rate: f64,
    pub buffer_utilization: f64,
}
```

These technical specifications provide the detailed interface definitions and implementation details needed for integrating Claude-Flow into the vibe-kanban system, ensuring robust streaming, error handling, and monitoring capabilities.