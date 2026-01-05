read README.md 
do a git ls-files


TASKS:

in the project the user can select different coding agents to use, but i would like to be able to select claude-flow as an agent(swarm) to solve the tasks. please examine how to do that, make a plan, save it to ./plans/ as md  including mermaid flowcharts and then implment and test the feature end to end and with 100% unit testing and 100% code coverage.

Use json outputs to get the agent information from the claude-flow agents. Research claoude flow here : https://github.com/ruvnet/claude-flow

its installed on the server and can be used via npx. 

make use of the json streaming to capture agents outputs as to present on the vibe-kanban board frontend/store in backend like normal like for the other agents, run  in non interactive mode. 

test the feature using playwright mcp server simulating normal user stories for the new feature
any research check the contex7 mcp server or websearch etc. 
all playwright tests shall save their test artifacts to the ./testartefacts/ folder during the tests

END OF TASKS.

======
info below ONLY:

Based on the **claude-flow** repository documentation, here are the key pages where you can find comprehensive information about JSON streaming output configuration and agent chaining:

## 📄 **Primary Documentation Pages**

### 1. **[Stream-JSON Chaining Wiki](https://github.com/ruvnet/claude-flow/wiki/Stream-Chaining)** 🔗
**This is the MAIN documentation page** for everything related to JSON streaming output and agent chaining.

**Key sections include:**
- **Core Configuration**: How to enable streaming with `--output-format stream-json` and `--input-format stream-json`
- **Automatic Chaining**: How Claude-Flow automatically pipes agent outputs when dependencies exist
- **Workflow Configuration**: JSON schema for configuring chaining in workflows
- **Implementation Details**: Code examples showing spawn configuration, flag management, and stream piping
- **Stream Format Specification**: Complete interface definition for StreamMessage objects
- **Debugging Guide**: How to monitor, debug, and troubleshoot stream chains

### 2. **[Non-Interactive Mode Guide](https://github.com/ruvnet/claude-flow/wiki/Non-Interactive-Mode)** 🤖
Covers streaming configuration for automation pipelines and CI/CD environments.

**Relevant content:**
- Stream-JSON chaining in headless environments
- Environment detection and automatic configuration
- JSON output formats for automation
- Docker and container usage with streaming

### 3. **[Automation Commands Reference](https://github.com/ruvnet/claude-flow/wiki/Automation-Commands)** 🎛️
Documents streaming options for automation commands like `mle-star`.

**Key options:**
- `--output-format stream-json`: Enables streaming output
- `--chaining`: Explicitly enable stream chaining
- `--no-chaining`: Disable stream chaining
- Non-interactive mode configuration

### 4. **[Workflow Orchestration Guide](https://github.com/ruvnet/claude-flow/wiki/Workflow-Orchestration)** 🔄
Explains chaining patterns and workflow configurations.

**Includes:**
- Linear, parallel, and conditional chaining patterns
- Workflow JSON configuration examples
- Benefits and best practices for chaining

## 🔧 **Quick Configuration Reference**

### **Basic Command-Line Usage**
```bash
# Enable JSON streaming output
claude-flow automation mle-star --dataset data.csv --target label --claude --output-format stream-json

# Manual chaining with Claude
claude --print --output-format stream-json "Task 1" | \
claude --print --input-format stream-json --output-format stream-json "Task 2" | \
claude --print --input-format stream-json "Task 3"
```

### **Workflow Configuration**
```json
{
  "name": "Stream Chaining Demo",
  "settings": {
    "enableChaining": true,
    "outputFormat": "stream-json"
  },
  "tasks": [
    {
      "id": "task1",
      "name": "Analyze Data",
      "assignTo": "agent1",
      "claudePrompt": "Analyze this data and output structured insights"
    },
    {
      "id": "task2",
      "name": "Process Results",
      "assignTo": "agent2",
      "depends": ["task1"],
      "claudePrompt": "You are receiving analysis results from the previous agent via stream-json..."
    }
  ]
}
```

## 📊 **Stream Format Specification**

The JSON streaming output uses **NDJSON** (newline-delimited JSON) with these message types:
- `init`: Session initialization
- `message`: Assistant/user messages
- `tool_use`: Tool invocations
- `tool_result`: Tool execution results
- `result`: Final task completion status

## 🎯 **Key Features**

- **Automatic Detection**: Chaining is enabled by default when using non-interactive mode with `stream-json` output format and task dependencies
- **Context Preservation**: 100% conversation history flows between agents (vs 60-70% with file-based methods)
- **Performance**: 40-60% faster than file-based handoffs with 95% latency reduction
- **Memory Efficient**: No intermediate file storage required

For complete implementation details, examples, and troubleshooting, visit the **[Stream-JSON Chaining Wiki](https://github.com/ruvnet/claude-flow/wiki/Stream-Chaining)** page.

=====

and here more information: https://github.com/ruvnet/claude-flow/wiki/Stream-Chaining
