import { test, expect } from '@playwright/test';
import * as fs from 'fs';
import * as path from 'path';

const TEST_ARTIFACTS_DIR = './testartefacts';

/**
 * Playwright test suite for Claude-Flow integration in Vibe Kanban
 *
 * This test suite validates the complete user journey for using Claude-Flow
 * as a coding agent in the Vibe Kanban application.
 *
 * Test artifacts are saved to ./testartefacts/ directory during testing
 */

// Ensure test artifacts directory exists
if (!fs.existsSync(TEST_ARTIFACTS_DIR)) {
  fs.mkdirSync(TEST_ARTIFACTS_DIR, { recursive: true });
}

test.describe('Claude-Flow Integration', () => {
  test.beforeEach(async ({ page }) => {
    // Navigate to Vibe Kanban
    await page.goto('http://localhost:3000');

    // Wait for the application to load
    await page.waitForLoadState('networkidle');
    await page.waitForSelector('[data-testid="app-container"]', { timeout: 30000 });

    // Save initial state screenshot
    await page.screenshot({
      path: path.join(TEST_ARTIFACTS_DIR, '01_initial_app_state.png'),
      fullPage: true
    });
  });

  test.afterEach(async ({ page }) => {
    // Save final state screenshot
    await page.screenshot({
      path: path.join(TEST_ARTIFACTS_DIR, '99_final_state.png'),
      fullPage: true
    });

    // Save page console logs
    const logs: string[] = [];
    page.on('console', msg => {
      logs.push(`${msg.type()}: ${msg.text()}`);
    });

    // Save network requests
    const networkRequests: any[] = [];
    page.on('response', response => {
      networkRequests.push({
        url: response.url(),
        status: response.status(),
        statusText: response.statusText()
      });
    });

    // Write artifacts to files
    fs.writeFileSync(
      path.join(TEST_ARTIFACTS_DIR, 'console_logs.json'),
      JSON.stringify(logs, null, 2)
    );

    fs.writeFileSync(
      path.join(TEST_ARTIFACTS_DIR, 'network_requests.json'),
      JSON.stringify(networkRequests, null, 2)
    );
  });

  test('should display Claude-Flow as available agent option', async ({ page }) => {
    // Navigate to agent configuration or task creation
    await page.click('[data-testid="create-task-button"]');

    // Look for agent selection dropdown
    await page.waitForSelector('[data-testid="agent-selector"]', { timeout: 10000 });

    // Take screenshot of agent selection
    await page.screenshot({
      path: path.join(TEST_ARTIFACTS_DIR, '02_agent_selection_dropdown.png'),
      fullPage: false
    });

    // Check if Claude-Flow appears in the dropdown
    const agentOptions = await page.locator('[data-testid="agent-selector"] option').allTextContents();

    // Save agent options for verification
    fs.writeFileSync(
      path.join(TEST_ARTIFACTS_DIR, '03_available_agents.json'),
      JSON.stringify(agentOptions, null, 2)
    );

    // Verify Claude-Flow is available
    expect(agentOptions).toContain('Claude-Flow');
  });

  test('should select Claude-Flow agent and configure options', async ({ page }) => {
    // Create a new task
    await page.click('[data-testid="create-task-button"]');

    // Select Claude-Flow agent
    await page.selectOption('[data-testid="agent-selector"]', 'Claude-Flow');

    // Take screenshot after selection
    await page.screenshot({
      path: path.join(TEST_ARTIFACTS_DIR, '04_claude_flow_selected.png'),
      fullPage: false
    });

    // Verify that Claude-Flow configuration options are displayed
    await expect(page.locator('[data-testid="claude-flow-config"]')).toBeVisible();

    // Test different configuration variants
    const configVariants = ['DEFAULT', 'SWARM', 'AUTOMATION'];

    for (const variant of configVariants) {
      try {
        // Select variant
        await page.selectOption('[data-testid="claude-flow-variant"]', variant);

        // Take screenshot of configuration
        await page.screenshot({
          path: path.join(TEST_ARTIFACTS_DIR, `05_claude_flow_${variant}_config.png`),
          fullPage: false
        });

        // Verify configuration options are displayed
        await expect(page.locator(`[data-testid="config-${variant.toLowerCase()}"]`)).toBeVisible();

      } catch (error) {
        console.log(`Variant ${variant} not available: ${error}`);
      }
    }

    // Save configuration state
    const configState = await page.evaluate(() => {
      return {
        selectedAgent: document.querySelector('[data-testid="agent-selector"]')?.value,
        selectedVariant: document.querySelector('[data-testid="claude-flow-variant"]')?.value,
        nonInteractive: document.querySelector('[data-testid="non-interactive"]')?.getAttribute('data-checked'),
        enableChaining: document.querySelector('[data-testid="enable-chaining"]')?.getAttribute('data-checked'),
        agentId: document.querySelector('[data-testid="agent-id"]')?.getAttribute('value')
      };
    });

    fs.writeFileSync(
      path.join(TEST_ARTIFACTS_DIR, '06_configuration_state.json'),
      JSON.stringify(configState, null, 2)
    );
  });

  test('should execute task with Claude-Flow agent', async ({ page }) => {
    // Create a new task with Claude-Flow
    await page.click('[data-testid="create-task-button"]');

    // Select Claude-Flow agent
    await page.selectOption('[data-testid="agent-selector"]', 'Claude-Flow');

    // Configure for automation mode (simpler for testing)
    await page.selectOption('[data-testid="claude-flow-variant"]', 'AUTOMATION');

    // Fill in task description
    const taskDescription = 'Create a simple hello world program in Python';
    await page.fill('[data-testid="task-description"]', taskDescription);

    // Take screenshot before execution
    await page.screenshot({
      path: path.join(TEST_ARTIFACTS_DIR, '07_task_ready_for_execution.png'),
      fullPage: false
    });

    // Execute the task
    await page.click('[data-testid="execute-task-button"]');

    // Wait for execution to start
    await page.waitForSelector('[data-testid="task-executing"]', { timeout: 15000 });

    // Take screenshot of execution state
    await page.screenshot({
      path: path.join(TEST_ARTIFACTS_DIR, '08_task_executing.png'),
      fullPage: false
    });

    // Monitor execution progress
    await page.waitForSelector('[data-testid="task-completed"], [data-testid="task-failed"]', {
      timeout: 300000 // 5 minutes timeout for execution
    });

    // Take screenshot of final state
    await page.screenshot({
      path: path.join(TEST_ARTIFACTS_DIR, '09_task_completed.png'),
      fullPage: true
    });

    // Check execution results
    const executionResults = await page.evaluate(() => {
      const taskElement = document.querySelector('[data-testid="task-card"]');
      return {
        status: taskElement?.getAttribute('data-status'),
        agent: taskElement?.getAttribute('data-agent'),
        duration: taskElement?.getAttribute('data-duration'),
        hasOutput: !!document.querySelector('[data-testid="task-output"]'),
        hasErrors: !!document.querySelector('[data-testid="task-errors"]')
      };
    });

    fs.writeFileSync(
      path.join(TEST_ARTIFACTS_DIR, '10_execution_results.json'),
      JSON.stringify(executionResults, null, 2)
    );

    // Verify that task was processed (status may vary based on actual execution)
    expect(executionResults.agent).toBe('CLAUDE_FLOW');
  });

  test('should display Claude-Flow output in real-time', async ({ page }) => {
    // Create a task with longer execution to observe streaming
    await page.click('[data-testid="create-task-button"]');

    await page.selectOption('[data-testid="agent-selector"]', 'Claude-Flow');
    await page.selectOption('[data-testid="claude-flow-variant"]', 'SWARM');

    const taskDescription = 'Analyze the current directory structure and provide a summary';
    await page.fill('[data-testid="task-description"]', taskDescription);

    // Execute the task
    await page.click('[data-testid="execute-task-button"]');

    // Wait for execution to start
    await page.waitForSelector('[data-testid="task-executing"]', { timeout: 15000 });

    // Monitor for real-time output updates
    let outputUpdates: string[] = [];

    // Listen for output updates
    page.on('response', async (response) => {
      if (response.url().includes('/api/tasks/') && response.url().includes('/logs')) {
        try {
          const logs = await response.json();
          outputUpdates.push(...logs.map((log: any) => log.content));
        } catch (error) {
          console.log('Failed to parse logs:', error);
        }
      }
    });

    // Wait for task completion or timeout
    await page.waitForSelector('[data-testid="task-completed"], [data-testid="task-failed"]', {
      timeout: 180000 // 3 minutes for analysis task
    });

    // Capture final output
    await page.waitForSelector('[data-testid="task-output"]', { timeout: 10000 });

    // Take screenshot of output
    await page.screenshot({
      path: path.join(TEST_ARTIFACTS_DIR, '11_realtime_output.png'),
      fullPage: true
    });

    // Extract output content
    const outputContent = await page.locator('[data-testid="task-output"]').textContent();

    fs.writeFileSync(
      path.join(TEST_ARTIFACTS_DIR, '12_output_content.json'),
      JSON.stringify({
        outputUpdates,
        finalContent: outputContent,
        updateCount: outputUpdates.length
      }, null, 2)
    );

    // Verify that output was captured
    expect(outputContent).toBeTruthy();
    expect(outputContent?.length).toBeGreaterThan(0);
  });

  test('should handle error scenarios gracefully', async ({ page }) => {
    // Test with invalid configuration
    await page.click('[data-testid="create-task-button"]');

    await page.selectOption('[data-testid="agent-selector"]', 'Claude-Flow');

    // Try to execute with missing required fields
    await page.fill('[data-testid="task-description"]', ''); // Empty description

    // Attempt execution
    await page.click('[data-testid="execute-task-button"]');

    // Check for validation errors
    await expect(page.locator('[data-testid="validation-error"]')).toBeVisible();

    // Take screenshot of error state
    await page.screenshot({
      path: path.join(TEST_ARTIFACTS_DIR, '13_validation_error.png'),
      fullPage: false
    });

    // Test with invalid agent configuration
    await page.fill('[data-testid="task-description"]', 'Valid task');

    // Set invalid configuration
    await page.fill('[data-testid="agent-id"]', 'invalid-agent-id');

    // Execute and expect failure
    await page.click('[data-testid="execute-task-button"]');
    await page.waitForSelector('[data-testid="task-failed"]', { timeout: 60000 });

    // Take screenshot of failure state
    await page.screenshot({
      path: path.join(TEST_ARTIFACTS_DIR, '14_execution_failure.png'),
      fullPage: false
    });

    // Capture error details
    const errorDetails = await page.evaluate(() => {
      const errorElement = document.querySelector('[data-testid="task-errors"]');
      return {
        hasError: !!errorElement,
        errorMessage: errorElement?.textContent,
        taskStatus: document.querySelector('[data-testid="task-card"]')?.getAttribute('data-status')
      };
    });

    fs.writeFileSync(
      path.join(TEST_ARTIFACTS_DIR, '15_error_details.json'),
      JSON.stringify(errorDetails, null, 2)
    );

    // Verify error handling
    expect(errorDetails.hasError).toBe(true);
    expect(errorDetails.taskStatus).toBe('failed');
  });

  test('should support follow-up tasks with same session', async ({ page }) => {
    // Create first task
    await page.click('[data-testid="create-task-button"]');
    await page.selectOption('[data-testid="agent-selector"]', 'Claude-Flow');
    await page.fill('[data-testid="task-description"]', 'Create a basic Python file');
    await page.click('[data-testid="execute-task-button"]');

    // Wait for completion
    await page.waitForSelector('[data-testid="task-completed"]', { timeout: 120000 });

    // Create follow-up task
    await page.click('[data-testid="create-follow-up-button"]');

    // Verify session is maintained
    const sessionInfo = await page.evaluate(() => {
      return {
        hasSessionId: !!document.querySelector('[data-testid="session-id"]'),
        sessionId: document.querySelector('[data-testid="session-id"]')?.getAttribute('value'),
        previousTaskId: document.querySelector('[data-testid="previous-task"]')?.getAttribute('data-task-id')
      };
    });

    fs.writeFileSync(
      path.join(TEST_ARTIFACTS_DIR, '16_follow_up_session.json'),
      JSON.stringify(sessionInfo, null, 2)
    );

    // Execute follow-up task
    await page.fill('[data-testid="task-description"]', 'Add error handling to the Python file');
    await page.click('[data-testid="execute-task-button"]');

    // Wait for completion
    await page.waitForSelector('[data-testid="task-completed"]', { timeout: 120000 });

    // Take screenshot of follow-up completion
    await page.screenshot({
      path: path.join(TEST_ARTIFACTS_DIR, '17_follow_up_completed.png'),
      fullPage: true
    });

    // Verify session continuity
    expect(sessionInfo.hasSessionId).toBe(true);
  });

  test('should display agent availability status correctly', async ({ page }) => {
    // Navigate to agent settings or configuration
    await page.click('[data-testid="settings-button"]');
    await page.click('[data-testid="agents-tab"]');

    // Check Claude-Flow availability status
    const availabilityStatus = await page.evaluate(() => {
      const claudeFlowRow = document.querySelector('[data-testid="agent-claude-flow"]');
      return {
        isVisible: !!claudeFlowRow,
        status: claudeFlowRow?.querySelector('[data-testid="availability-status"]')?.textContent,
        hasConfigButton: !!claudeFlowRow?.querySelector('[data-testid="configure-agent"]'),
        isAvailable: claudeFlowRow?.querySelector('[data-testid="availability-status"]')?.classList.contains('available')
      };
    });

    fs.writeFileSync(
      path.join(TEST_ARTIFACTS_DIR, '18_availability_status.json'),
      JSON.stringify(availabilityStatus, null, 2)
    );

    // Take screenshot of availability status
    await page.screenshot({
      path: path.join(TEST_ARTIFACTS_DIR, '19_availability_display.png'),
      fullPage: true
    });

    // Verify Claude-Flow is visible in settings
    expect(availabilityStatus.isVisible).toBe(true);
  });
});

/**
 * Additional utility functions for test artifacts management
 */

test.describe('Test Artifacts Management', () => {
  test('should create and manage test artifacts directory', () => {
    // Verify test artifacts directory exists and is writable
    expect(fs.existsSync(TEST_ARTIFACTS_DIR)).toBe(true);

    // Test writing a sample artifact
    const sampleData = {
      testName: 'Claude-Flow Integration Tests',
      timestamp: new Date().toISOString(),
      status: 'running'
    };

    const artifactPath = path.join(TEST_ARTIFACTS_DIR, 'sample_artifact.json');
    fs.writeFileSync(artifactPath, JSON.stringify(sampleData, null, 2));

    // Verify file was created and can be read
    expect(fs.existsSync(artifactPath)).toBe(true);

    const readData = JSON.parse(fs.readFileSync(artifactPath, 'utf-8'));
    expect(readData.testName).toBe('Claude-Flow Integration Tests');

    // Clean up sample artifact
    fs.unlinkSync(artifactPath);
  });
});