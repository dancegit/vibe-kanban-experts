# Claude-Flow Integration Mermaid Diagrams

## System Architecture Overview

```mermaid
graph TB
    subgraph "Frontend Layer"
        UI[User Interface]
        AgentSelector[Agent Selector]
        TaskDialog[Task Creation Dialog]
        LogViewer[Real-time Log Viewer]
        StateManager[State Management]
    end

    subgraph "API Gateway Layer"
        Router[Axum Router]
        Middleware[Authentication Middleware]
        RateLimit[Rate Limiting]
    end

    subgraph "Service Layer"
        TaskService[Task Service]
        ExecutorService[Executor Service]
        SessionService[Session Service]
        MCPService[MCP Service]
    end

    subgraph "Agent Orchestration Layer"
        AgentRegistry[Agent Registry]
        ProcessManager[Process Manager]
        StreamProcessor[Stream Processor]
        LogAggregator[Log Aggregator]
    end

    subgraph "Claude-Flow Integration"
        CFExecutor[ClaudeFlow Executor]
        JSONStream[JSON Stream Parser]
        CommandBuilder[Command Builder]
        AgentChainer[Agent Chainer]
        StateTracker[State Tracker]
    end

    subgraph "Storage Layer"
        SQLite[(SQLite Database)]
        LogStore[Log Storage]
        Cache[(Redis Cache)]
        FileSystem[File System]
    end

    subgraph "External Systems"
        NPM[NPM Registry]
        ClaudeFlowCLI[Claude-Flow CLI]
        ClaudeAPI[Claude API]
        FileIO[File I/O]
    end

    UI --> AgentSelector
    UI --> TaskDialog
    UI --> LogViewer
    StateManager --> Router

    Router --> TaskService
    Router --> ExecutorService
    Router --> SessionService
    Router --> MCPService

    TaskService --> AgentRegistry
    ExecutorService --> ProcessManager
    SessionService --> LogAggregator

    ProcessManager --> CFExecutor
    StreamProcessor --> JSONStream
    LogAggregator --> StateTracker

    CFExecutor --> CommandBuilder
    CFExecutor --> JSONStream
    CFExecutor --> AgentChainer

    CommandBuilder --> ClaudeFlowCLI
    JSONStream --> LogStore
    StateTracker --> Cache

    LogViewer --> SQLite
    LogViewer --> LogStore

    ClaudeFlowCLI --> ClaudeAPI
    CommandBuilder --> FileIO
```

## Detailed Data Flow

### Task Creation Flow

```mermaid
sequenceDiagram
    participant User
    participant Frontend
    participant API
    participant TaskService
    participant ExecutorService
    participant CFExecutor
    participant ClaudeFlow
    participant Database

    User->>Frontend: Select Claude-Flow Agent
    Frontend->>API: POST /api/tasks
    API->>TaskService: Create Task
    TaskService->>Database: Save Task
    Database-->>TaskService: Task Created
    TaskService-->>API: Task Response
    API-->>Frontend: Task Created

    User->>Frontend: Create Attempt
    Frontend->>API: POST /api/task-attempts
    API->>ExecutorService: Initialize Executor
    ExecutorService->>CFExecutor: New Instance
    CFExecutor->>ClaudeFlow: Spawn Process
    ClaudeFlow-->>CFExecutor: Process Started
    CFExecutor-->>API: Process Created
    API-->>Frontend: Attempt Started

    Note over ClaudeFlow: Non-interactive mode
    Note over ClaudeFlow: Stream JSON enabled
```

### JSON Streaming Flow

```mermaid
sequenceDiagram
    participant CFExecutor
    participant JSONStream
    participant Process
    participant LogStore
    participant Frontend
    participant Database

    Process->>JSONStream: Raw Stream Data
    JSONStream->>JSONStream: Parse NDJSON
    JSONStream->>LogStore: Store Logs
    JSONStream->>Frontend: WebSocket Update
    JSONStream->>Database: Update Status

    alt Message Type: init
        JSONStream->>LogStore: Session Initialized
    else Message Type: message
        JSONStream->>LogStore: Assistant Message
    else Message Type: tool_use
        JSONStream->>LogStore: Tool Invocation
    else Message Type: tool_result
        JSONStream->>LogStore: Tool Result
    else Message Type: result
        JSONStream->>Database: Execution Complete
        JSONStream->>LogStore: Final Summary
    end

    Frontend->>Frontend: Update UI
```

## Component Interaction Diagrams

### Claude-Flow Executor Architecture

```mermaid
graph TB
    subgraph "ClaudeFlow Executor Core"
        CE[ClaudeFlow Executor]
        CM[Command Manager]
        PM[Process Manager]
        SP[Stream Processor]
        EH[Error Handler]
    end

    subgraph "Configuration Management"
        CFG[Config Parser]
        ENV[Environment Manager]
        VAL[Validator]
    end

    subgraph "Stream Processing"
        NP[NDJSON Parser]
        FM[Frame Manager]
        BM[Buffer Manager]
    end

    subgraph "State Management"
        SM[State Machine]
        TS[Task State]
        AS[Agent State]
    end

    CE --> CM
    CE --> PM
    CE --> SP
    CE --> EH

    CM --> CFG
    CFG --> VAL
    VAL --> ENV

    SP --> NP
    NP --> FM
    FM --> BM

    PM --> SM
    SM --> TS
    SM --> AS
```

### Multi-Agent Orchestration

```mermaid
graph LR
    subgraph "Workflow Definition"
        WF[Workflow JSON]
        TD[Task Dependencies]
        AG[Agent Groups]
    end

    subgraph "Execution Engine"
        CE[Coordinator Executor]
        AE1[Agent Executor 1]
        AE2[Agent Executor 2]
        AE3[Agent Executor N]
    end

    subgraph "Agent Types"
        C1[Claude-Flow Agent]
        C2[Claude Code Agent]
        C3[Cursor Agent]
        Cx[Custom Agents]
    end

    WF --> CE
    TD --> CE
    AG --> CE

    CE --> AE1
    CE --> AE2
    CE --> AE3

    AE1 --> C1
    AE2 --> C2
    AE3 --> C3
```

## Database Schema Evolution

### Current Schema

```mermaid
erDiagram
    TASKS ||--o{ TASK_ATTEMPTS : has
    TASK_ATTEMPTS ||--o{ EXECUTION_PROCESSES : contains
    EXECUTION_PROCESSES ||--o{ EXECUTION_PROCESS_LOGS : generates
    SESSIONS ||--o{ EXECUTION_PROCESSES : manages

    TASKS {
        uuid id
        uuid project_id
        string title
        text description
        string status
        datetime created_at
        datetime updated_at
    }

    TASK_ATTEMPTS {
        uuid id
        uuid task_id
        string executor_type
        string status
        datetime created_at
    }

    EXECUTION_PROCESSES {
        uuid id
        uuid session_id
        string executor_action
        string status
        integer exit_code
        datetime started_at
        datetime completed_at
    }

    SESSIONS {
        uuid id
        uuid workspace_id
        string executor
        datetime created_at
    }
```

### Proposed Schema Extensions

```mermaid
erDiagram
    TASKS ||--o{ TASK_ATTEMPTS : has
    TASK_ATTEMPTS ||--o{ EXECUTION_PROCESSES : contains
    EXECUTION_PROCESSES ||--o{ EXECUTION_PROCESS_LOGS : generates
    SESSIONS ||--o{ EXECUTION_PROCESSES : manages

    %% New tables for Claude-Flow
    CLAUDE_FLOW_CONFIGS ||--o{ TASK_ATTEMPTS : configures
    CLAUDE_FLOW_WORKFLOWS ||--o{ TASK_ATTEMPTS : defines
    STREAM_MESSAGES ||--o{ EXECUTION_PROCESS_LOGS : streams
    AGENT_STATES ||--o{ EXECUTION_PROCESSES : tracks

    CLAUDE_FLOW_CONFIGS {
        uuid id
        uuid task_attempt_id
        json output_format
        boolean non_interactive
        boolean chaining
        text append_prompt
        json custom_flags
        datetime created_at
    }

    CLAUDE_FLOW_WORKFLOWS {
        uuid id
        uuid task_attempt_id
        json workflow_definition
        json agent_assignments
        text status
        datetime created_at
        datetime completed_at
    }

    STREAM_MESSAGES {
        uuid id
        uuid execution_process_id
        string message_type
        json message_data
        text raw_content
        datetime timestamp
    }

    AGENT_STATES {
        uuid id
        uuid execution_process_id
        string agent_id
        json state_data
        text status
        datetime updated_at
    }
```

## API Flow Diagrams

### Task Execution Flow

```mermaid
sequenceDiagram
    participant Client
    participant API
    participant ExecutorManager
    participant ClaudeFlow
    participant StreamHandler
    participant Database

    Client->>API: POST /api/task-attempts
    Note right of API: Body: {<br/>task_id: "uuid",<br/>executor: "CLAUDE_FLOW",<br/>config: {...}
    API->>ExecutorManager: Create Executor
    ExecutorManager->>ClaudeFlow: Initialize
    ClaudeFlow->>Database: Save Config
    ClaudeFlow-->>ExecutorManager: Executor Ready

    API-->>Client: 201 Created
    Note right of API: Response: {<br/>attempt_id: "uuid",<br/>status: "pending"}

    par Process Execution
        ExecutorManager->>ClaudeFlow: Start Process
        ClaudeFlow->>ClaudeFlow: Build Command
        ClaudeFlow->>ClaudeFlow: Spawn Child Process
    and Stream Processing
        ClaudeFlow->>StreamHandler: Emit Stream Data
        StreamHandler->>Database: Store Logs
        StreamHandler->>Client: WebSocket Event
    end

    ClaudeFlow-->>ExecutorManager: Process Complete
    ExecutorManager->>Database: Update Status
    Database-->>ExecutorManager: Status Updated
    ExecutorManager-->>Client: WebSocket Final
```

### WebSocket Real-time Updates

```mermaid
sequenceDiagram
    participant Client
    participant WebSocket
    participant StreamHandler
    participant JSONParser
    participant Process

    Client->>WebSocket: Connect
    WebSocket-->>Client: Connected

    loop JSON Stream Processing
        Process->>JSONParser: Emit Data
        JSONParser->>JSONParser: Parse NDJSON
        JSONParser->>StreamHandler: Valid Message
        StreamHandler->>StreamHandler: Format Response
        StreamHandler->>WebSocket: Send Update
        WebSocket->>Client: Message Event
    end

    Client->>WebSocket: Disconnect
    WebSocket-->>Client: Disconnected
```

## Error Handling Flow

```mermaid
graph TB
    subgraph "Error Detection"
        E1[Stream Parse Error]
        E2[Process Error]
        E3[Command Error]
        E4[Network Error]
    end

    subgraph "Error Handling"
        H1[Parse Retry Logic]
        H2[Graceful Degradation]
        H3[Fallback Commands]
        H4[Connection Retry]
    end

    subgraph "User Feedback"
        U1[Toast Notifications]
        U2[Log Display]
        U3[Status Updates]
        U4[Retry Options]
    end

    E1 --> H1
    E2 --> H2
    E3 --> H3
    E4 --> H4

    H1 --> U1
    H2 --> U2
    H3 --> U3
    H4 --> U4
```

## Performance Optimization Flow

```mermaid
graph TB
    subgraph "Input Optimization"
        I1[Batch Processing]
        I2[Buffer Management]
        I3[Compression]
    end

    subgraph "Processing Optimization"
        P1[Async Processing]
        P2[Parallel Streams]
        P3[Memory Pool]
    end

    subgraph "Output Optimization"
        O1[Delta Updates]
        O2[Rate Limiting]
        O3[Caching]
    end

    I1 --> P1
    I2 --> P2
    I3 --> P3

    P1 --> O1
    P2 --> O2
    P3 --> O3
```

## Security Architecture

```mermaid
graph TB
    subgraph "Input Validation"
        V1[Command Sanitization]
        V2[Path Validation]
        V3[JSON Schema Check]
        V4[Rate Limit Check]
    end

    subgraph "Execution Sandbox"
        S1[Process Isolation]
        S2[Resource Limits]
        S3[File System Restriction]
        S4[Network Segmentation]
    end

    subgraph "Output Filtering"
        F1[Content Sanitization]
        F2[PII Detection]
        F3[Log Redaction]
        F4[Audit Trail]
    end

    V1 --> S1
    V2 --> S2
    V3 --> S3
    V4 --> S4

    S1 --> F1
    S2 --> F2
    S3 --> F3
    S4 --> F4
```

## Monitoring & Observability

```mermaid
graph LR
    subgraph "Metrics Collection"
        MC1[Execution Time]
        MC2[Memory Usage]
        MC3[CPU Usage]
        MC4[Stream Latency]
        MC5[Error Rate]
    end

    subgraph "Metrics Processing"
        MP1[Aggregation]
        MP2[Threshold Detection]
        MP3[Alert Generation]
    end

    subgraph "Visualization"
        V1[Real-time Dashboard]
        V2[Historical Charts]
        V3[Alert Notifications]
    end

    MC1 --> MP1
    MC2 --> MP1
    MC3 --> MP1
    MC4 --> MP1
    MC5 --> MP1

    MP1 --> MP2
    MP2 --> MP3
    MP3 --> V1
    MP3 --> V2
    MP3 --> V3
```

## Testing Architecture

```mermaid
graph TB
    subgraph "Unit Tests"
        U1[JSON Parser Tests]
        U2[Command Builder Tests]
        U3[State Manager Tests]
        U4[Error Handler Tests]
    end

    subgraph "Integration Tests"
        I1[Process Spawn Tests]
        I2[Stream Processing Tests]
        I3[Database Integration Tests]
        I4[API Endpoint Tests]
    end

    subgraph "E2E Tests"
        E1[Playwright UI Tests]
        E2[User Workflow Tests]
        E3[Error Scenario Tests]
        E4[Performance Tests]
    end

    U1 --> I1
    U2 --> I2
    U3 --> I3
    U4 --> I4

    I1 --> E1
    I2 --> E2
    I3 --> E3
    I4 --> E4
```

## Deployment Architecture

```mermaid
graph TB
    subgraph "Build Pipeline"
        B1[Compile Rust]
        B2[Build Frontend]
        B3[Run Tests]
        B4[Package Assets]
    end

    subgraph "Container Build"
        C1[Base Image]
        C2[Install Dependencies]
        C3[Copy Artifacts]
        C4[Configure Environment]
    end

    subgraph "Deployment"
        D1[Docker Registry]
        D2[Kubernetes Cluster]
        D3[Service Mesh]
        D4[Load Balancer]
    end

    B1 --> C1
    B2 --> C2
    B3 --> C3
    B4 --> C4

    C1 --> D1
    C2 --> D1
    C3 --> D1
    C4 --> D1

    D1 --> D2
    D2 --> D3
    D3 --> D4
```

These diagrams provide a comprehensive visual representation of the Claude-Flow integration architecture, showing all key components, data flows, and system interactions required for a successful implementation.