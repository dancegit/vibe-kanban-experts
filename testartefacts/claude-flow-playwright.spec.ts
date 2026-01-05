import { test, expect, Page, Browser } from '@playwright/test';
import path from 'path';

const ARTIFACTS_DIR = path.join(__dirname, 'testartefacts');

// Test configuration and helpers
test.describe('Claude-Flow Integration', () => {
  let page: Page;
  let browser: Browser;

  test.beforeAll(async ({ browser: browserInstance }) => {
    browser = browserInstance;
  });

  test.beforeEach(async ({ page: pageInstance }) => {
    page = pageInstance;
    // Navigate to the application
    await page.goto('http://localhost:3000', { waitUntil: 'networkidle' });
  });

  test.afterEach(async () => {
    // Take screenshot after each test
    await page.screenshot({
      path: `${ARTIFACTS_DIR}/screenshot-${Date.now()}.png`,
      fullPage: true,
    });
  });

  test.afterAll(async () => {
    // Close browser and generate report
    if (browser) {
      await browser.close();
    }
  });

  test.describe('Agent Selection', () => {
    test('should display claude-flow in agent selection dropdown', async () => {
      // Navigate to agent selection
      await page.click('[data-testid="agent-selector"]');

      // Check if Claude-Flow option is visible
      const claudeFlowOption = page.locator('text=ClaudeFlow');
      await expect(claudeFlowOption).toBeVisible();

      // Take screenshot
      await page.screenshot({
        path: `${ARTIFACTS_DIR}/agent-selection-claude-flow.png`,
        fullPage: true,
      });
    });

    test('should allow selecting claude-flow agent', async () => {
      // Open agent selector
      await page.click('[data-testid="agent-selector"]');

      // Select Claude-Flow
      await page.click('text=ClaudeFlow');

      // Verify it's selected
      const selectedAgent = page.locator('[data-testid="selected-agent"]');
      await expect(selectedAgent).toContainText('ClaudeFlow');

      // Take screenshot
      await page.screenshot({
        path: `${ARTIFACTS_DIR}/claude-flow-selected.png`,
        fullPage: true,
      });
    });

    test('should show claude-flow specific configuration options', async () => {
      // Select Claude-Flow agent
      await page.click('[data-testid="agent-selector"]');
      await page.click('text=ClaudeFlow');

      // Check for claude-flow specific options
      await expect(page.locator('label:has-text("Non-interactive Mode")')).toBeVisible();
      await expect(page.locator('label:has-text("Enable Chaining")')).toBeVisible();
      await expect(page.locator('input[name="agent_id"]')).toBeVisible();
      await expect(page.locator('input[name="workflow_file"]')).toBeVisible();

      // Take screenshot
      await page.screenshot({
        path: `${ARTIFACTS_DIR}/claude-flow-config.png`,
        fullPage: true,
      });
    });
  });

  test.describe('Claude-Flow Configuration', () => {
    test.beforeEach(async () => {
      // Select Claude-Flow agent
      await page.click('[data-testid="agent-selector"]');
      await page.click('text=ClaudeFlow');
    });

    test('should enable non-interactive mode', async () => {
      // Enable non-interactive mode
      await page.check('input[name="non_interactive"]');

      // Verify it's checked
      const checkbox = page.locator('input[name="non_interactive"]');
      await expect(checkbox).toBeChecked();

      // Take screenshot
      await page.screenshot({
        path: `${ARTIFACTS_DIR}/non-interactive-enabled.png`,
        fullPage: true,
      });
    });

    test('should enable chaining', async () => {
      // Enable chaining
      await page.check('input[name="enable_chaining"]');

      // Verify it's checked
      const checkbox = page.locator('input[name="enable_chaining"]');
      await expect(checkbox).toBeChecked();

      // Take screenshot
      await page.screenshot({
        path: `${ARTIFACTS_DIR}/chaining-enabled.png`,
        fullPage: true,
      });
    });

    test('should accept agent ID input', async () => {
      // Enter agent ID
      await page.fill('input[name="agent_id"]', 'coding-agent');

      // Verify input
      const input = page.locator('input[name="agent_id"]');
      await expect(input).toHaveValue('coding-agent');

      // Take screenshot
      await page.screenshot({
        path: `${ARTIFACTS_DIR}/agent-id-input.png`,
        fullPage: true,
      });
    });

    test('should accept workflow file path', async () => {
      // Enter workflow file path
      await page.fill('input[name="workflow_file"]', '/path/to/workflow.json');

      // Verify input
      const input = page.locator('input[name="workflow_file"]');
      await expect(input).toHaveValue('/path/to/workflow.json');

      // Take screenshot
      await page.screenshot({
        path: `${ARTIFACTS_DIR}/workflow-file-input.png`,
        fullPage: true,
      });
    });

    test('should accept task description', async () => {
      // Enter task description
      await page.fill('input[name="task_description"]', 'Analyze and refactor code');

      // Verify input
      const input = page.locator('input[name="task_description"]');
      await expect(input).toHaveValue('Analyze and refactor code');

      // Take screenshot
      await page.screenshot({
        path: `${ARTIFACTS_DIR}/task-description-input.png`,
        fullPage: true,
      });
    });
  });

  test.describe('Task Execution', () => {
    test.beforeEach(async () => {
      // Select Claude-Flow agent and configure it
      await page.click('[data-testid="agent-selector"]');
      await page.click('text=ClaudeFlow');

      // Configure for non-interactive execution
      await page.check('input[name="non_interactive"]');
      await page.check('input[name="enable_chaining"]');
      await page.fill('input[name="agent_id"]', 'coding-agent');
    });

    test('should execute task with claude-flow', async () => {
      // Enter task prompt
      await page.fill('textarea[name="task-prompt"]', 'Create a simple function to calculate fibonacci numbers');

      // Start execution
      await page.click('[data-testid="start-task"]');

      // Wait for execution to start
      await expect(page.locator('[data-testid="execution-status"]')).toContainText('Running');

      // Wait for some output
      await page.waitForSelector('[data-testid="output-stream"]', { timeout: 10000 });

      // Take screenshot during execution
      await page.screenshot({
        path: `${ARTIFACTS_DIR}/execution-in-progress.png`,
        fullPage: true,
      });

      // Wait for completion (with timeout)
      await expect(page.locator('[data-testid="execution-status"]')).toContainText('Completed', { timeout: 30000 });

      // Take screenshot after completion
      await page.screenshot({
        path: `${ARTIFACTS_DIR}/execution-completed.png`,
        fullPage: true,
      });
    });

    test('should display JSON streaming output', async () => {
      // Enter task prompt
      await page.fill('textarea[name="task-prompt"]', 'Write a unit test');

      // Start execution
      await page.click('[data-testid="start-task"]');

      // Wait for streaming output
      await page.waitForSelector('[data-testid="output-stream"]', { timeout: 10000 });

      // Check for JSON streaming messages
      const output = page.locator('[data-testid="output-stream"]');
      await expect(output).toContainText('type');

      // Take screenshot
      await page.screenshot({
        path: `${ARTIFACTS_DIR}/json-streaming-output.png`,
        fullPage: true,
      });
    });

    test('should show tool usage in real-time', async () => {
      // Enter task prompt that will use tools
      await page.fill('textarea[name="task-prompt"]', 'Create a new file called test.txt and write hello world');

      // Start execution
      await page.click('[data-testid="start-task"]');

      // Wait for execution
      await page.waitForSelector('[data-testid="tool-usage"]', { timeout: 15000 });

      // Check for tool usage indicators
      const toolUsage = page.locator('[data-testid="tool-usage"]');
      await expect(toolUsage).toBeVisible();

      // Take screenshot
      await page.screenshot({
        path: `${ARTIFACTS_DIR}/tool-usage-realtime.png`,
        fullPage: true,
      });
    });
  });

  test.describe('Error Handling', () => {
    test('should handle claude-flow not installed', async () => {
      // Select Claude-Flow agent
      await page.click('[data-testid="agent-selector"]');
      await page.click('text=ClaudeFlow');

      // Try to execute task
      await page.fill('textarea[name="task-prompt"]', 'Test task');
      await page.click('[data-testid="start-task"]');

      // Wait for error message
      await expect(page.locator('[data-testid="error-message"]')).toContainText('not found', { timeout: 10000 });

      // Take screenshot
      await page.screenshot({
        path: `${ARTIFACTS_DIR}/error-not-installed.png`,
        fullPage: true,
      });
    });

    test('should handle invalid workflow file', async () => {
      // Select Claude-Flow agent
      await page.click('[data-testid="agent-selector"]');
      await page.click('text=ClaudeFlow');

      // Enter invalid workflow file path
      await page.fill('input[name="workflow_file"]', '/nonexistent/workflow.json');

      // Try to execute task
      await page.fill('textarea[name="task-prompt"]', 'Test task');
      await page.click('[data-testid="start-task"]');

      // Wait for error or fallback behavior
      await page.waitForTimeout(5000);

      // Take screenshot
      await page.screenshot({
        path: `${ARTIFACTS_DIR}/error-invalid-workflow.png`,
        fullPage: true,
      });
    });

    test('should handle network timeout', async () => {
      // Select Claude-Flow agent
      await page.click('[data-testid="agent-selector"]');
      await page.click('text=ClaudeFlow');

      // Enter task prompt
      await page.fill('textarea[name="task-prompt"]', 'Long running task');

      // Start execution
      await page.click('[data-testid="start-task"]');

      // Wait for timeout
      await page.waitForSelector('[data-testid="timeout-message"]', { timeout: 30000 });

      // Take screenshot
      await page.screenshot({
        path: `${ARTIFACTS_DIR}/error-timeout.png`,
        fullPage: true,
      });
    });
  });

  test.describe('User Stories', () => {
    test('complete user story: selecting claude-flow and running a task', async () => {
      // User opens the application
      await expect(page.locator('h1')).toContainText('Vibe Kanban');

      // User clicks on agent selector
      await page.click('[data-testid="agent-selector"]');

      // User selects Claude-Flow from dropdown
      await page.click('text=ClaudeFlow');

      // User sees configuration options
      await expect(page.locator('label:has-text("Non-interactive Mode")')).toBeVisible();

      // User enables non-interactive mode and chaining
      await page.check('input[name="non_interactive"]');
      await page.check('input[name="enable_chaining"]');

      // User enters agent ID
      await page.fill('input[name="agent_id"]', 'my-coding-agent');

      // User enters task prompt
      await page.fill('textarea[name="task-prompt"]', 'Create a REST API endpoint for user management');

      // User clicks start task
      await page.click('[data-testid="start-task"]');

      // User sees execution status
      await expect(page.locator('[data-testid="execution-status"]')).toContainText('Running');

      // User waits for completion
      await expect(page.locator('[data-testid="execution-status"]')).toContainText('Completed', { timeout: 30000 });

      // User sees the output
      await expect(page.locator('[data-testid="output-stream"]')).toBeVisible();

      // Take final screenshot
      await page.screenshot({
        path: `${ARTIFACTS_DIR}/user-story-complete.png`,
        fullPage: true,
      });
    });

    test('user story: configuring claude-flow for automation', async () => {
      // User opens application
      await page.goto('http://localhost:3000', { waitUntil: 'networkidle' });

      // User selects Claude-Flow
      await page.click('[data-testid="agent-selector"]');
      await page.click('text=ClaudeFlow');

      // User configures for automation
      await page.check('input[name="non_interactive"]');
      await page.check('input[name="enable_chaining"]');
      await page.fill('input[name="agent_id"]', 'automation-agent');
      await page.fill('input[name="workflow_file"]', 'automation-workflow.json');
      await page.fill('input[name="task_description"]', 'Automated code review and refactoring');

      // User saves configuration
      await page.click('[data-testid="save-configuration"]');

      // User sees confirmation
      await expect(page.locator('[data-testid="save-confirmation"]')).toContainText('Configuration saved');

      // Take screenshot
      await page.screenshot({
        path: `${ARTIFACTS_DIR}/user-story-automation-config.png`,
        fullPage: true,
      });
    });

    test('user story: monitoring claude-flow output in real-time', async () => {
      // User selects Claude-Flow
      await page.click('[data-testid="agent-selector"]');
      await page.click('text=ClaudeFlow');

      // User enters a complex task
      await page.fill('textarea[name="task-prompt"]', 'Analyze the codebase and suggest improvements');

      // User starts execution
      await page.click('[data-testid="start-task"]');

      // User watches real-time output
      await page.waitForSelector('[data-testid="output-stream"]', { timeout: 10000 });

      // User sees streaming messages
      await expect(page.locator('[data-testid="output-stream"]')).toContainText('type');

      // User sees tool usage
      await page.waitForSelector('[data-testid="tool-usage"]', { timeout: 15000 });

      // User monitors progress
      await page.waitForSelector('[data-testid="progress-indicator"]', { timeout: 5000 });

      // Take screenshot
      await page.screenshot({
        path: `${ARTIFACTS_DIR}/user-story-realtime-monitoring.png`,
        fullPage: true,
      });
    });
  });

  test.describe('Performance and Load', () => {
    test('should handle multiple concurrent executions', async () => {
      // Select Claude-Flow
      await page.click('[data-testid="agent-selector"]');
      await page.click('text=ClaudeFlow');

      // Configure for multiple tasks
      await page.check('input[name="non_interactive"]');
      await page.fill('input[name="enable_chaining"]', 'true');

      // Start multiple tasks
      for (let i = 0; i < 3; i++) {
        await page.fill('textarea[name="task-prompt"]', `Task ${i + 1}: Analyze file ${i + 1}`);
        await page.click('[data-testid="start-task"]');

        // Wait a bit between starts
        await page.waitForTimeout(2000);
      }

      // Check that all are running
      const runningCount = await page.locator('[data-testid="execution-status"]:has-text("Running")').count();
      expect(runningCount).toBeGreaterThan(0);

      // Take screenshot
      await page.screenshot({
        path: `${ARTIFACTS_DIR}/multiple-concurrent-executions.png`,
        fullPage: true,
      });
    });

    test('should handle large output efficiently', async () => {
      // Select Claude-Flow
      await page.click('[data-testid="agent-selector"]');
      await page.click('text=ClaudeFlow');

      // Enter a task that will produce large output
      await page.fill('textarea[name="task-prompt"]', 'Generate detailed documentation for all functions in the codebase');

      // Start execution
      await page.click('[data-testid="start-task"]');

      // Wait for output
      await page.waitForSelector('[data-testid="output-stream"]', { timeout: 10000 });

      // Check that output is being streamed efficiently (no memory issues)
      const outputElement = page.locator('[data-testid="output-stream"]');
      await expect(outputElement).toBeVisible();

      // Take screenshot
      await page.screenshot({
        path: `${ARTIFACTS_DIR}/large-output-handling.png`,
        fullPage: true,
      });
    });
  });
});

// Custom test for screenshots and artifacts
test('capture console logs and network requests', async ({ page }) => {
  // Navigate to app
  await page.goto('http://localhost:3000', { waitUntil: 'networkidle' });

  // Capture console messages
  const consoleMessages: string[] = [];
  page.on('console', msg => {
    consoleMessages.push(`${msg.type()}: ${msg.text()}`);
  });

  // Capture network requests
  const networkRequests: any[] = [];
  page.on('response', response => {
    networkRequests.push({
      url: response.url(),
      status: response.status(),
      contentType: response.headers()['content-type']
    });
  });

  // Interact with claude-flow features
  await page.click('[data-testid="agent-selector"]');
  await page.click('text=ClaudeFlow');

  // Save console logs
  const fs = require('fs');
  fs.writeFileSync(
    `${ARTIFACTS_DIR}/console-logs.json`,
    JSON.stringify(consoleMessages, null, 2)
  );

  // Save network requests
  fs.writeFileSync(
    `${ARTIFACTS_DIR}/network-requests.json`,
    JSON.stringify(networkRequests, null, 2)
  );

  // Take final screenshot
  await page.screenshot({
    path: `${ARTIFACTS_DIR}/final-state.png`,
    fullPage: true,
  });
});
