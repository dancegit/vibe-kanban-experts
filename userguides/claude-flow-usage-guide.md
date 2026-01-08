# Claude-Flow Usage Guide

## Overview

Claude-Flow is an AI-powered development orchestration tool that extends the capabilities of coding agents like Claude Code, Gemini CLI, and Codex. It provides a centralized platform for managing multiple AI coding agents, orchestrating their tasks, and tracking their progress through an intuitive Kanban-style interface.

## Prerequisites

Before using Claude-Flow, ensure you have the following installed:

### Required Software

1. **Node.js** (>= 18.0.0)
   - Install from: https://nodejs.org/
   - Verify installation: `node --version`

2. **pnpm** (>= 8.0.0)
   - Install: `npm install -g pnpm`
   - Verify installation: `pnpm --version`

3. **Rust** (latest stable)
   - Install from: https://rustup.rs/
   - Verify installation: `cargo --version`

### Optional (Recommended) Development Tools

```bash
cargo install cargo-watch
cargo install sqlx-cli
```

## Installation

### Option 1: From npm registry (when published)

```bash
npx claude-flow
```

### Option 2: Local Development Build

1. Clone the repository:
   ```bash
   git clone https://github.com/your-repo/claude-flow.git
   cd claude-flow
   ```

2. Run the build script:
   ```bash
   ./build-claude-flow.sh
   ```

3. Install locally:
   ```bash
   cd npx-cli
   npm link
   claude-flow
   ```

### Option 3: Install from tarball

```bash
cd npx-cli
npm install -g claude-flow-0.0.1-alpha.0.tgz
claude-flow
```

## Usage

### Basic Commands

```bash
# Start Claude-Flow
npx claude-flow

# Start in MCP (Model Context Protocol) mode
npx claude-flow --mcp

# Start in review mode for code reviews
npx claude-flow review
```

### Environment Variables

Configure Claude-Flow using these environment variables:

| Variable | Type | Default | Description |
|----------|------|---------|-------------|
| `PORT` | Runtime | Auto-assign | Server port (auto-assigned in dev) |
| `BACKEND_PORT` | Runtime | Auto | Backend port (dev mode only) |
| `FRONTEND_PORT` | Runtime | 3000 | Frontend dev server port |
| `HOST` | Runtime | 127.0.0.1 | Backend server host |
| `POSTHOG_API_KEY` | Build-time | Empty | PostHog analytics key |
| `POSTHOG_API_ENDPOINT` | Build-time | Empty | PostHog endpoint |

Example:
```bash
PORT=4000 BACKEND_PORT=4001 npx claude-flow
```

### Remote Deployment

When running Claude-Flow on a remote server:

1. **Access via tunnel**: Use Cloudflare Tunnel, ngrok, or similar
2. **Configure SSH**: Set in Settings → Editor Integration
3. **Prerequisites**:
   - SSH access from local to remote
   - Passwordless SSH authentication
   - VSCode Remote-SSH extension

## Features

### 1. Multi-Agent Orchestration
- Switch between different coding agents seamlessly
- Run multiple agents in parallel or sequence
- Track progress of all active agents

### 2. Kanban-Style Task Management
- Visual task organization
- Drag-and-drop task management
- Real-time status updates

### 3. Centralized Configuration
- Unified MCP (Model Context Protocol) config management
- Agent-specific settings
- Global configuration options

### 4. Code Review Integration
- Integrated code review workflows
- Support for multiple review agents
- Automated review processes

### 5. Remote Development Support
- SSH-based remote project access
- Cloud deployment ready
- Docker support

## Architecture

Claude-Flow consists of:

1. **Frontend**: React + TypeScript + Vite + Tailwind CSS
2. **Backend**: Rust server with SQLx database
3. **NPX Wrapper**: Node.js CLI for easy distribution
4. **MCP Integration**: Model Context Protocol support

### Project Structure

```
vibe-kanban-experts/
├── frontend/          # React frontend
├── crates/           # Rust backend crates
│   ├── server/       # Main server
│   ├── db/          # Database models
│   └── ...          # Other crates
├── npx-cli/         # NPX wrapper
├── shared/          # Shared TypeScript types
└── docs/            # Documentation
```

## Development

### Running in Development Mode

```bash
# Start both frontend and backend
pnpm run dev

# Frontend only
pnpm run frontend:dev

# Backend only (with watch mode)
pnpm run backend:dev:watch
```

### Building from Source

```bash
# Full build
./local-build.sh

# Frontend only
cd frontend && pnpm build

# Backend only
cargo build --release
```

### Testing

```bash
# Frontend checks
pnpm run check
pnpm run lint

# Backend tests
cargo test --workspace
```

## Troubleshooting

### Common Issues

1. **Port already in use**
   - Set different ports: `PORT=4000 npx claude-flow`

2. **Binary not found**
   - Rebuild: `./build-claude-flow.sh`
   - Clear cache: `rm -rf ~/.vibe-kanban-cache`

3. **Permission denied**
   - Make script executable: `chmod +x build-claude-flow.sh`
   - Check file permissions in npx-cli/bin/

4. **Build failures**
   - Ensure all dependencies are installed
   - Check Rust toolchain is up to date
   - Verify Node.js version >= 18

### Debug Mode

Enable debug mode for verbose output:
```bash
VIBE_KANBAN_DEBUG=1 npx claude-flow
```

## Advanced Usage

### Custom Agent Integration

1. Configure agent in MCP settings
2. Add agent-specific configurations
3. Set up authentication
4. Configure task templates

### API Usage

Claude-Flow provides REST APIs for:
- Task management
- Agent control
- Configuration management
- Status monitoring

### Plugin Development

Extend Claude-Flow with custom plugins:
1. Implement MCP-compatible interface
2. Register with Claude-Flow
3. Configure in settings
4. Use in workflows

## Support

- **Documentation**: https://docs.claude-flow.com
- **Issues**: https://github.com/your-repo/claude-flow/issues
- **Discussions**: https://github.com/your-repo/claude-flow/discussions
- **Discord**: https://discord.gg/claude-flow

## Contributing

1. Fork the repository
2. Create a feature branch
3. Make your changes
4. Run tests
5. Submit a pull request

See [CONTRIBUTING.md](CONTRIBUTING.md) for detailed guidelines.

## License

MIT License - see [LICENSE](LICENSE) for details.