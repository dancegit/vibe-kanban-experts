# Vibe Kanban Docker Deployment Plan

## Overview
Deploy Vibe Kanban as a Docker container using Dokploy with production-ready configuration.

## Project Structure Analysis
- **Main Application**: Rust backend (`server` binary) + React frontend
- **Database**: PostgreSQL (required)
- **Build Process**: Multi-stage Docker build (Node.js for frontend, Rust for backend)
- **Runtime**: Alpine Linux with compiled server binary

## Deployment Architecture
- **Strategy**: Single Docker container with PostgreSQL database
- **Proxy**: Traefik (managed by Dokploy)
- **Domain**: `vibe-kanban-dev.<domain>` for development branch
- **Git Integration**: Automated builds on git push

## Phase 1: Create Dokploy Project and Application

### 1.1 Create Project
- Project Name: `vibe-kanban`
- Description: Vibe Kanban - AI Coding Agent Orchestration Platform

### 1.2 Create Application
- Application Name: `vibe-kanban-dev`
- Build Type: Dockerfile
- Dockerfile Path: `/Dockerfile`
- Docker Context: `/`
- Docker Build Stage: `runtime`

### 1.3 Configure Git Provider
- Repository: Connect to GitHub repo
- Branch: `dev`
- Auto-deploy: Enabled

## Phase 2: Database Setup

### 2.1 Create PostgreSQL Database
- Database Name: `vibe-kanban-db-dev`
- Database User: `vibe_kanban_dev`
- Docker Image: `postgres:15-alpine`

### 2.2 Database Configuration
- Enable connection pooling
- Set appropriate memory limits
- Configure backups if needed

## Phase 3: Environment Variables

### 3.1 Build-time Variables (Build Args)
```env
POSTHOG_API_KEY=              # Optional: PostHog analytics key
POSTHOG_API_ENDPOINT=         # Optional: PostHog endpoint
```

### 3.2 Runtime Variables
```env
# Server Configuration
HOST=0.0.0.0
PORT=3000

# Database
DATABASE_URL=postgres://vibe_kanban_dev:${DB_PASSWORD}@dokploy-host:5432/vibe_kanban_db_dev

# GitHub OAuth (Required for GitHub integration)
GITHUB_OAUTH_CLIENT_ID=
GITHUB_OAUTH_CLIENT_SECRET=

# GitHub App (Optional, for advanced features)
GITHUB_APP_ID=
GITHUB_APP_PRIVATE_KEY=
GITHUB_APP_WEBHOOK_SECRET=
GITHUB_APP_SLUG=

# Google OAuth (Optional)
GOOGLE_OAUTH_CLIENT_ID=
GOOGLE_OAUTH_CLIENT_SECRET=

# JWT Secret (Required)
VIBEKANBAN_REMOTE_JWT_SECRET=

# Email (Optional)
LOOPS_EMAIL_API_KEY=

# Analytics (Optional)
POSTHOG_API_KEY=
POSTHOG_API_ENDPOINT=

# Cloud Storage (Optional, for review features)
R2_ACCESS_KEY_ID=
R2_SECRET_ACCESS_KEY=
R2_REVIEW_ENDPOINT=
R2_REVIEW_BUCKET=
REVIEW_WORKER_BASE_URL=

# Additional optional features
DISABLE_WORKTREE_ORPHAN_CLEANUP=  # Set to disable cleanup
```

## Phase 4: Domain Configuration

### 4.1 Create Domain
- Domain: `vibe-kanban-dev.<your-domain.com>`
- HTTPS: Enabled (Let's Encrypt)
- Path: `/`
- Strip Path: No

## Phase 5: Deployment Configuration

### 5.1 Health Checks
- Endpoint: `http://localhost:3000`
- Interval: 30s
- Timeout: 3s
- Start Period: 5s
- Retries: 3

### 5.2 Resource Limits
- CPU Reservation: 0.5
- CPU Limit: 2.0
- Memory Reservation: 512MB
- Memory Limit: 2GB

### 5.3 Restart Policy
- Policy: `unless-stopped`
- Max Attempts: 3

## Phase 6: Git Workflow

### 6.1 Branch Strategy
- `main`: Production branch (application: `vibe-kanban-prod`)
- `dev`: Development branch (application: `vibe-kanban-dev`)

### 6.2 Subdomain Mapping
- Production: `vibe-kanban.<domain>` → `main` branch
- Development: `vibe-kanban-dev.<domain>` → `dev` branch

## Phase 7: Post-Deployment Verification

### 7.1 Verify Application Health
- Check container logs for successful startup
- Verify database migrations run
- Test authentication flows

### 7.2 Monitor Performance
- Check resource usage
- Verify response times
- Monitor error rates

## Configuration Files Needed

### docker-compose.yml (Alternative to Dockerfile)
```yaml
version: '3.8'
services:
  vibe-kanban:
    build:
      context: .
      dockerfile: Dockerfile
      args:
        - POSTHOG_API_KEY
        - POSTHOG_API_ENDPOINT
    environment:
      - HOST=0.0.0.0
      - PORT=3000
      - DATABASE_URL
      - GITHUB_OAUTH_CLIENT_ID
      - GITHUB_OAUTH_CLIENT_SECRET
      - GITHUB_APP_ID
      - GITHUB_APP_PRIVATE_KEY
      - GITHUB_APP_WEBHOOK_SECRET
      - GITHUB_APP_SLUG
      - GOOGLE_OAUTH_CLIENT_ID
      - GOOGLE_OAUTH_CLIENT_SECRET
      - VIBEKANBAN_REMOTE_JWT_SECRET
      - LOOPS_EMAIL_API_KEY
      - POSTHOG_API_KEY
      - POSTHOG_API_ENDPOINT
      - R2_ACCESS_KEY_ID
      - R2_SECRET_ACCESS_KEY
      - R2_REVIEW_ENDPOINT
      - R2_REVIEW_BUCKET
      - REVIEW_WORKER_BASE_URL
    ports:
      - "3000:3000"
    restart: unless-stopped
    healthcheck:
      test: ["CMD", "wget", "--spider", "-q", "http://localhost:3000"]
      interval: 30s
      timeout: 3s
      retries: 3
      start_period: 5s
```

## Security Considerations

1. **Secrets Management**: Use Dokploy secret management for sensitive data
2. **Network Isolation**: Place in isolated Docker network
3. **File Permissions**: Ensure proper file permissions in container
4. **CORS**: Configure appropriate CORS settings
5. **Rate Limiting**: Implement rate limiting for API endpoints

## Monitoring & Alerts

1. **Application Metrics**:
   - Response time
   - Error rate
   - Active connections

2. **Infrastructure Metrics**:
   - CPU usage
   - Memory usage
   - Disk space
   - Network I/O

3. **Alert Conditions**:
   - High error rate (>5%)
   - High response time (>2s)
   - High resource usage (>80%)
   - Application down

## Rollback Plan

1. **Database**: Keep migration history for rollback
2. **Application**: Tag Docker images with version numbers
3. **Configuration**: Store configs in git for version control
4. **Procedure**:
   - Stop new container
   - Start previous version container
   - Rollback database if needed

## Next Steps

1. Create swarmtasks directory entry for this deployment
2. Document actual environment variables in deployment report
3. Set up monitoring and alerting
4. Configure log aggregation
5. Set up backup strategies for database
6. Document troubleshooting procedures