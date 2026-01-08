# Vibe Kanban Production Environment Deployment Report

**Deployment Date**: 2026-01-08
**Environment**: Production
**Branch**: main
**Status**: ✅ Configuration Complete

## Overview

This document describes the production deployment setup for Vibe Kanban using Dokploy with separate docker-compose configurations for different environments.

## Docker Compose Configuration

### Production: docker-compose.prod.yml
- **Location**: `/docker-compose.prod.yml`
- **Purpose**: Production-ready configuration
- **Includes**: Application service only (PostgreSQL managed by Dokploy)
- **Branch**: `main`
- **Features**:
  - Resource limits (CPU: 0.5-2.0 cores, Memory: 512M-2G)
  - Production logging (info level)
  - Traefik labels for routing
  - Health checks enabled

### Development: docker-compose.dev.yml
- **Location**: `/docker-compose.dev.yml`
- **Purpose**: Development configuration
- **Includes**: Application service only (PostgreSQL managed by Dokploy)
- **Branch**: `dev`
- **Features**:
  - Debug logging enabled
  - No resource limits (flexible development)
  - Health checks enabled

**Notes**:
- Both files exclude PostgreSQL since Dokploy manages databases separately
- Dokploy provides PostgreSQL as a managed service
- Environment variables handle database connection strings

## Dokploy Project Configuration

### Project Details
- **Project ID**: `yajLDneAt7tcYkcVGILvs`
- **Project Name**: `vibe-kanban`
- **Description**: Vibe Kanban - AI Coding Agent Orchestration Platform

### Environment
All applications run in the default "production" environment within Dokploy:
- **Environment ID**: `pUz-S1nIVamEExv6jtFgK`

## Production Application: vibe-kanban-prod

### Application Details
- **Application ID**: `cEgJ5CAUDw1oeKacJOhLm`
- **Name**: `vibe-kanban-prod`
- **App Name**: `app-input-back-end-application-sob9df`
- **Build Type**: Docker Compose (docker-compose.prod.yml)
- **Git Branch**: `main`
- **Auto-deploy**: Enabled

### Build Configuration
- **Docker Compose File**: `docker-compose.prod.yml`
- **Docker Context**: `/`
- **Build Arguments**:
  ```yaml
  POSTHOG_API_KEY: (optional)
  POSTHOG_API_ENDPOINT: (optional)
  ```

### Runtime Configuration
```yaml
HOST: 0.0.0.0
PORT: 3000
DATABASE_URL: postgres://vibe_kanban_prod:prod_db_password_2026_secure_change_me@dokploy-host:5432/vibe_kanban_prod
RUST_LOG: info
```

### Domains (Production)

1. **Primary Domain**
   - **Domain ID**: `AYpbbg21oj8Mw-yIzz2lF`
   - **Domain**: `vibe-kanban.aboutco.ai`
   - **HTTPS**: Enabled (Let's Encrypt)
   - **Certificate**: Letsencrypt

2. **Short Domain**
   - **Domain ID**: `6TfQzJgiObJi__4O7cUE3`
   - **Domain**: `vk.aboutco.ai`
   - **HTTPS**: Enabled (Let's Encrypt)
   - **Certificate**: Letsencrypt

3. **WWW Domain**
   - **Domain ID**: `FjdHKE6-1mJM4nMT3uXfq`
   - **Domain**: `www.vibe-kanban.aboutco.ai`
   - **HTTPS**: Enabled (Let's Encrypt)
   - **Certificate**: Letsencrypt

### Resource Configuration
- **CPU Reservation**: 0.5 cores (via docker-compose.yml)
- **CPU Limit**: 2.0 cores (via docker-compose.yml)
- **Memory Reservation**: 512MB (via docker-compose.yml)
- **Memory Limit**: 2GB (via docker-compose.yml)
- **Restart Policy**: `unless-stopped`

### Health Check Configuration
```yaml
test: ["CMD", "wget", "--spider", "-q", "http://localhost:3000/health"]
interval: 30s
timeout: 10s
retries: 3
start_period: 40s
```

## Development Application: vibe-kanban-dev

### Application Details
- **Application ID**: `MOO9hOxgOvvvid3WPSNMw`
- **Name**: `vibe-kanban-dev`
- **App Name**: `app-compress-wireless-hard-drive-vmmly9`
- **Build Type**: Docker Compose (docker-compose.dev.yml)
- **Git Branch**: `dev`
- **Auto-deploy**: Enabled

### Build Configuration
- **Docker Compose File**: `docker-compose.dev.yml`
- **Docker Context**: `/`
- **Build Arguments**:
  ```yaml
  POSTHOG_API_KEY: (optional)
  POSTHOG_API_ENDPOINT: (optional)
  ```

### Runtime Configuration
```yaml
HOST: 0.0.0.0
PORT: 3000
DATABASE_URL: postgres://vibe_kanban_dev:dev_db_password_2026_secure@dokploy-host:5432/vibe_kanban_dev
RUST_LOG: debug
```

### Domains (Development)

1. **Local Development Domain**
   - **Domain ID**: `CC_W9EbHYU6yXzRkMy1Zi`
   - **Domain**: `vibe-kanban-dev.localhost`
   - **HTTPS**: Enabled (Let's Encrypt)
   - **Certificate**: Letsencrypt

2. **Dev Subdomain**
   - **Domain ID**: `nTZpy5KlASVb7jL0FeCbA`
   - **Domain**: `vibe-kanban-dev.aboutco.ai`
   - **HTTPS**: Enabled (Let's Encrypt)
   - **Certificate**: Letsencrypt

3. **Alternate Dev Domain**
   - **Domain ID**: `DrJ3qack2C3ZNYzD0wEfB`
   - **Domain**: `dev.vibe-kanban.aboutco.ai`
   - **HTTPS**: Enabled (Let's Encrypt)
   - **Certificate**: Letsencrypt

4. **Short Dev Domain**
   - **Domain ID**: `HcrdDFpxKAnXBYlQEQDoD`
   - **Domain**: `vk-dev.aboutco.ai`
   - **HTTPS**: Enabled (Let's Encrypt)
   - **Certificate**: Letsencrypt

### Resource Configuration
- **CPU Reservation**: No limit (flexible development)
- **CPU Limit**: No limit (flexible development)
- **Memory Reservation**: No limit (flexible development)
- **Memory Limit**: No limit (flexible development)
- **Restart Policy**: `unless-stopped`

### Health Check Configuration
```yaml
test: ["CMD", "wget", "--spider", "-q", "http://localhost:3000/health"]
interval: 30s
timeout: 10s
retries: 3
start_period: 40s
```

## Database Configuration

### Production Database: vibe-kanban-db-prod
- **Database ID**: `oBsNQCiOCwEWlMhgJ6H05`
- **Name**: `vibe-kanban-db-prod`
- **App Name**: `vibekanbandbprod-qajmjq`
- **Database**: `vibe_kanban_prod`
- **User**: `vibe_kanban_prod`
- **Password**: `prod_db_password_2026_secure_change_me`
- **Docker Image**: `postgres:15`
- **Status**: Idle (ready for deployment)

### Development Database: vibe-kanban-db-dev
- **Database ID**: `StaSfKP0puzWq7XqqRmH8`
- **Name**: `vibe-kanban-db-dev`
- **App Name**: `vibekanbandbdev-hijdjx`
- **Database**: `vibe_kanban_dev`
- **User**: `vibe_kanban_dev`
- **Password**: `dev_db_password_2026_secure`
- **Docker Image**: `postgres:15-alpine`
- **Status**: Idle (ready for deployment)

## Required Environment Variables

Both environments require the same set of OAuth and configuration variables. These should be configured in Dokploy:

### Essential Variables (Required)
```bash
# GitHub OAuth
GITHUB_OAUTH_CLIENT_ID=         # Required for GitHub integration
GITHUB_OAUTH_CLIENT_SECRET=     # Required for GitHub integration

# JWT Secret (Generate with: openssl rand -hex 32)
VIBEKANBAN_REMOTE_JWT_SECRET=   # Required for authentication
```

### Optional Variables
```bash
# GitHub App (for advanced GitHub features)
GITHUB_APP_ID=
GITHUB_APP_PRIVATE_KEY=
GITHUB_APP_WEBHOOK_SECRET=
GITHUB_APP_SLUG=

# Google OAuth
GOOGLE_OAUTH_CLIENT_ID=
GOOGLE_OAUTH_CLIENT_SECRET=

# Email Service
LOOPS_EMAIL_API_KEY=

# Analytics
POSTHOG_API_KEY=
POSTHOG_API_ENDPOINT=

# Cloud Storage (for review features)
R2_ACCESS_KEY_ID=
R2_SECRET_ACCESS_KEY=
R2_REVIEW_ENDPOINT=
R2_REVIEW_BUCKET=
REVIEW_WORKER_BASE_URL=
```

## Git Workflow

### Branch Strategy
1. **Development Branch**: `dev`
   - Docker Compose: `docker-compose.dev.yml`
   - Database: `vibe-kanban-db-dev`
   - Application: `vibe-kanban-dev`
   - Domains: Dev subdomains of aboutco.ai

2. **Production Branch**: `main`
   - Docker Compose: `docker-compose.prod.yml`
   - Database: `vibe-kanban-db-prod`
   - Application: `vibe-kanban-prod`
   - Domains: Production domains of aboutco.ai

### Deployment Process
1. **Development**: Push to `dev` → Auto-deploys to dev environment
2. **Production**: Merge `dev` → `main` → Auto-deploys to production
3. **Configuration**: Each branch has its own docker-compose file

## Access URLs

### Production URLs
- **Main**: https://vibe-kanban.aboutco.ai
- **Short**: https://vk.aboutco.ai
- **WWW**: https://www.vibe-kanban.aboutco.ai

### Development URLs
- **Main Dev**: https://vibe-kanban-dev.aboutco.ai
- **Alternate**: https://dev.vibe-kanban.aboutco.ai
- **Short**: https://vk-dev.aboutco.ai
- **Local**: http://vibe-kanban-dev.localhost

## Deployment Steps

### Completed ✅
1. ✅ Create Dokploy project: `vibe-kanban`
2. ✅ Create docker-compose files (dev and prod)
3. ✅ Create PostgreSQL databases (dev and prod)
4. ✅ Create applications (dev and prod)
5. ✅ Configure GitHub integration for both applications
6. ✅ Configure domains for both environments
7. ✅ Push docker-compose files to GitHub (dev branch)

### Next Steps Required

#### Before First Deployment
1. **Configure Required Secrets** in Dokploy:
   - `GITHUB_OAUTH_CLIENT_ID` and `GITHUB_OAUTH_CLIENT_SECRET`
   - `VIBEKANBAN_REMOTE_JWT_SECRET` (generate with `openssl rand -hex 32`)

2. **Optional Secrets**:
   - Configure any optional OAuth providers
   - Set up analytics keys
   - Configure cloud storage if needed

3. **Merge to Main Branch**:
   - Create `main` branch if it doesn't exist
   - Merge `dev` to `main` to include docker-compose.prod.yml
   - Push `main` branch to GitHub

#### First Deployment
1. **Development**:
   - Ensure you're on `dev` branch
   - Push changes to trigger deployment
   - Monitor deployment logs in Dokploy

2. **Production**:
   - Merge `dev` → `main`
   - Push `main` branch to trigger deployment
   - Monitor deployment logs in Dokploy

3. **Verify Health**:
   - Check application logs for successful startup
   - Verify database migrations complete
   - Test all configured domains
   - Verify health endpoints return 200 OK

## Security Considerations

### Current Security Status
- ✅ All domains use HTTPS with Let's Encrypt
- ✅ Non-root user in Docker containers
- ✅ Separate databases for dev and prod
- ✅ Database credentials managed by Dokploy
- ⚠️ OAuth credentials need to be configured
- ⚠️ JWT secret needs to be generated and set

### Recommendations
1. **Generate Strong Secrets**: Use `openssl rand -hex 32` for JWT secret
2. **OAuth Configuration**: Set up GitHub OAuth for authentication
3. **Access Control**: Consider VPN or IP restrictions for admin access
4. **Monitoring**: Set up alerts for failed deployments
5. **Backups**: Configure automated database backups
6. **SSL**: All domains automatically get Let's Encrypt certificates

## Monitoring & Troubleshooting

### Health Checks
- **Endpoint**: `http://localhost:3000/health`
- **Method**: HTTP GET
- **Success**: Returns HTTP 200
- **Interval**: Every 30 seconds

### Application Logs
```bash
# View application logs via Dokploy dashboard or:
docker logs app-input-back-end-application-sob9df  # Production
docker logs app-compress-wireless-hard-drive-vmmly9  # Development
```

### Database Connection Test
```bash
# Production
docker exec -it vibekanbandbprod-qajmjq psql -U vibe_kanban_prod -d vibe_kanban_prod

# Development
docker exec -it vibekanbandbdev-hijdjx psql -U vibe_kanban_dev -d vibe_kanban_dev
```

### Common Issues

1. **Build Failures**:
   - Check Docker Compose syntax
   - Ensure all environment variables are set
   - Verify GitHub repository access

2. **Database Connection Errors**:
   - Verify DATABASE_URL format
   - Check database service health
   - Ensure network connectivity

3. **Domain Not Accessible**:
   - Check Traefik proxy status
   - Verify SSL certificate issuance
   - Confirm application health checks pass

## Resource Usage

### Production
- **CPU**: 0.5 - 2.0 cores
- **Memory**: 512MB - 2GB
- **Database**: Separate PostgreSQL 15 instance

### Development
- **CPU**: No limits (flexible)
- **Memory**: No limits (flexible)
- **Database**: Separate PostgreSQL 15-alpine instance

## Troubleshooting Guide

### Application Won't Start
1. Check Docker Compose file syntax: `docker-compose -f docker-compose.prod.yml config`
2. Verify all required environment variables are set
3. Check application logs for Rust compilation errors
4. Ensure OAuth secrets are properly configured

### Database Issues
1. Verify DATABASE_URL matches Dokploy database credentials
2. Check database health status in Dokploy
3. Ensure database migrations can run
4. Test database connectivity from application container

### Deployment Failures
1. Check build logs in Dokploy dashboard
2. Verify GitHub webhook is working
3. Ensure docker-compose file is valid
4. Check for missing environment variables

## Rollback Plan

### Application Rollback
1. Dokploy maintains previous deployment images
2. Use Dokploy dashboard to rollback to previous version
3. Or rebuild from previous Git commit

### Database Rollback
1. Keep migration history for rollback capability
2. Dokploy database backups (if configured)
3. Manual database dump before major changes

## Support

### Documentation
- [Vibe Kanban Documentation](https://vibekanban.com/docs)
- [Dokploy Documentation](https://dokploy.com/docs)
- [Docker Compose Reference](https://docs.docker.com/compose/)

### Repository
- **GitHub**: `https://github.com/dancegit/vibe-kanban-experts`
- **Dev Branch**: `dev` (uses docker-compose.dev.yml)
- **Main Branch**: `main` (uses docker-compose.prod.yml)

### Application Stack
- **Backend**: Rust (Axum/Actix-Web)
- **Frontend**: React + TypeScript
- **Database**: PostgreSQL 15
- **Build Tools**: Cargo + pnpm
- **Runtime**: Alpine Linux
- **Proxy**: Traefik

---

**Configuration prepared by**: Claude Code
**Project ID**: `yajLDneAt7tcYkcVGILvs`
**Status**: Ready for first deployment
**Last Updated**: 2026-01-08
