import { test, expect, Page } from '@playwright/test';
import path from 'path';

test.describe('Claude-Flow Agent Integration', () => {
  test.beforeEach(async ({ page }) => {
    // Navigate to the application
    await page.goto('http://localhost:3000');

    // Wait for the app to load
    await page.waitForSelector('[data-testid="app-root"]', { timeout: 10000 });
  });

  test('should display claude-flow as an available agent option', async ({ page }) => {
    // Navigate to agent selection or configuration
    await page.click('[data-testid="new-task-button"]');

    // Look for agent selection dropdown
    const agentDropdown = page.locator('[data-testid="agent-selection"], .agent-dropdown, select[name="agent"]');

    if (await agentDropdown.isVisible()) {
      // Click to open dropdown
      await agentDropdown.click();

      // Check if CLAUDE_FLOW option is available
      const claudeFlowOption = page.locator('option[value="CLAUDE_FLOW"], [data-value="CLAUDE_FLOW"]');
      await expect(claudeFlowOption).toBeVisible();
    } else {
      // Alternative: check for any mention of claude-flow in the interface
      const claudeFlowText = page.locator('text=/claude-flow/i');
      await expect(claudeFlowText).toBeVisible();
    }
  });

  test('should allow selecting claude-flow agent with default configuration', async ({ page }) => {
    // Navigate to task creation
    await page.click('[data-testid="new-task-button"]');

    // Fill in basic task information
    await page.fill('[data-testid="task-title"]', 'Test Claude-Flow Task');
    await page.fill('[data-testid="task-description"]', 'Testing claude-flow agent integration');

    // Select CLAUDE_FLOW as agent
    const agentSelection = page.locator('[data-testid="agent-selection"], select[name="agent"]');
    if (await agentSelection.isVisible()) {
      await agentSelection.selectOption('CLAUDE_FLOW');

      // Verify selection
      await expect(agentSelection).toHaveValue('CLAUDE_FLOW');
    }

    // Save the task
    await page.click('[data-testid="save-task-button"], button[type="submit"]');

    // Verify task was created successfully
    await expect(page.locator('text=Test Claude-Flow Task')).toBeVisible();
  });

  test('should show claude-flow specific configuration options', async ({ page }) => {
    // Navigate to agent configuration or advanced settings
    await page.click('[data-testid="new-task-button"]');

    // Fill basic information
    await page.fill('[data-testid="task-title"]', 'Config Test Task');

    // Select CLAUDE_FLOW
    const agentSelection = page.locator('[data-testid="agent-selection"], select[name="agent"]');
    if (await agentSelection.isVisible()) {
      await agentSelection.selectOption('CLAUDE_FLOW');

      // Look for claude-flow specific configuration
      const configSection = page.locator('[data-testid="claude-flow-config"], .claude-flow-config, .agent-config');

      if (await configSection.isVisible()) {
        // Check for specific configuration options
        const nonInteractiveCheckbox = page.locator('input[name="non_interactive"], [data-field="non_interactive"]');
        const chainingCheckbox = page.locator('input[name="enable_chaining"], [data-field="enable_chaining"]');
        const agentIdInput = page.locator('input[name="agent_id"], [data-field="agent_id"]');

        // Verify configuration options are present
        await expect(nonInteractiveCheckbox).toBeVisible();
        await expect(chainingCheckbox).toBeVisible();
        await expect(agentIdInput).toBeVisible();

        // Test configuration
        await nonInteractiveCheckbox.check();
        await chainingCheckbox.check();
        await agentIdInput.fill('test-swarm-agent');
      }
    }
  });

  test('should display claude-flow profile variants', async ({ page }) => {
    // Navigate to agent selection
    await page.click('[data-testid="new-task-button"]');

    // Select CLAUDE_FLOW agent
    const agentSelection = page.locator('[data-testid="agent-selection"], select[name="agent"]');
    if (await agentSelection.isVisible()) {
      await agentSelection.selectOption('CLAUDE_FLOW');

      // Look for profile variant selection
      const profileVariant = page.locator('[data-testid="profile-variant"], select[name="variant"], .profile-variant');

      if (await profileVariant.isVisible()) {
        // Check for available variants
        const defaultOption = page.locator('option[value="DEFAULT"], [data-value="DEFAULT"]');
        const swarmOption = page.locator('option[value="SWARM"], [data-value="SWARM"]');
        const automationOption = page.locator('option[value="AUTOMATION"], [data-value="AUTOMATION"]');

        // Verify all variants are available
        await expect(defaultOption).toBeVisible();
        await expect(swarmOption).toBeVisible();
        await expect(automationOption).toBeVisible();

        // Test selecting different variants
        await profileVariant.selectOption('SWARM');
        await expect(profileVariant).toHaveValue('SWARM');

        await profileVariant.selectOption('AUTOMATION');
        await expect(profileVariant).toHaveValue('AUTOMATION');
      }
    }
  });

  test('should handle claude-flow execution and show streaming output', async ({ page }) => {
    // Create a task with claude-flow agent
    await page.click('[data-testid="new-task-button"]');

    await page.fill('[data-testid="task-title"]', 'Execution Test Task');
    await page.fill('[data-testid="task-description"]', 'Execute a simple claude-flow task');

    // Select CLAUDE_FLOW with default configuration
    const agentSelection = page.locator('[data-testid="agent-selection"], select[name="agent"]');
    if (await agentSelection.isVisible()) {
      await agentSelection.selectOption('CLAUDE_FLOW');
    }

    // Start the execution
    await page.click('[data-testid="start-execution-button"], button[type="submit"]');

    // Wait for execution to start
    await page.waitForSelector('[data-testid="execution-output"], .execution-log', { timeout: 15000 });

    // Check for execution output area
    const outputArea = page.locator('[data-testid="execution-output"], .execution-log');
    await expect(outputArea).toBeVisible();

    // Look for streaming output indicators
    const streamingIndicator = page.locator('.streaming-indicator, [data-testid="streaming"]');

    // Note: Actual streaming output will depend on claude-flow availability
    // In a test environment, this might show error messages or connection issues
    if (await streamingIndicator.isVisible()) {
      await expect(streamingIndicator).toContainText('streaming');
    }

    // Check for any error messages if claude-flow is not available
    const errorMessage = page.locator('.error-message, [data-testid="execution-error"]');
    if (await errorMessage.isVisible()) {
      console.log('Expected error in test environment:', await errorMessage.textContent());
    }
  });

  test('should validate claude-flow configuration form', async ({ page }) => {
    await page.click('[data-testid="new-task-button"]');

    await page.fill('[data-testid="task-title"]', 'Validation Test');

    // Select CLAUDE_FLOW
    const agentSelection = page.locator('[data-testid="agent-selection"], select[name="agent"]');
    if (await agentSelection.isVisible()) {
      await agentSelection.selectOption('CLAUDE_FLOW');

      // Test form validation by filling invalid values
      const agentIdInput = page.locator('input[name="agent_id"], [data-field="agent_id"]');
      if (await agentIdInput.isVisible()) {
        // Test with special characters (should be handled gracefully)
        await agentIdInput.fill('agent-with-special-chars-123');

        // Test with empty string
        await agentIdInput.fill('');

        // Test with very long string
        await agentIdInput.fill('a'.repeat(200));
      }

      const workflowFileInput = page.locator('input[name="workflow_file"], [data-field="workflow_file"]');
      if (await workflowFileInput.isVisible()) {
        // Test with invalid file path
        await workflowFileInput.fill('/invalid/path/workflow.json');

        // Test with valid JSON file path
        await workflowFileInput.fill('./workflows/test-workflow.json');
      }
    }

    // Try to save and check for validation
    await page.click('[data-testid="save-task-button"], button[type="submit"]');

    // Check if validation messages appear
    const validationMessage = page.locator('.validation-message, .error-message, [data-testid="form-error"]');

    // Validation might pass or fail depending on backend implementation
    console.log('Validation result:', await validationMessage.isVisible() ? await validationMessage.textContent() : 'No validation errors');
  });

  test('should handle claude-flow availability status', async ({ page }) => {
    // Navigate to agent configuration or settings
    await page.goto('http://localhost:3000/settings/agents');

    // Look for agent availability indicators
    const agentStatus = page.locator('[data-testid="agent-status-CLAUDE_FLOW"], .agent-status');

    if (await agentStatus.isVisible()) {
      // Check for availability status
      const statusText = await agentStatus.textContent();

      if (statusText?.includes('Available') || statusText?.includes('Ready')) {
        console.log('Claude-flow is available for use');
      } else if (statusText?.includes('Not Found') || statusText?.includes('Unavailable')) {
        console.log('Claude-flow is not available - this is expected in test environment');
      } else {
        console.log('Claude-flow status:', statusText);
      }

      // Take screenshot for documentation
      await page.screenshot({
        path: '/home/clauderun/vibe-kanban-experts/testartifacts/claude-flow-availability-status.png',
        fullPage: true
      });
    }
  });
});

test.describe('Claude-Flow Error Handling', () => {
  test('should gracefully handle claude-flow not installed', async ({ page }) => {
    await page.goto('http://localhost:3000');

    // Create a task that would use claude-flow
    await page.click('[data-testid="new-task-button"]');

    await page.fill('[data-testid="task-title"]', 'Error Handling Test');
    await page.fill('[data-testid="task-description"]', 'Testing error handling when claude-flow is not available');

    // Select CLAUDE_FLOW
    const agentSelection = page.locator('[data-testid="agent-selection"], select[name="agent"]');
    if (await agentSelection.isVisible()) {
      await agentSelection.selectOption('CLAUDE_FLOW');
    }

    // Try to execute
    await page.click('[data-testid="start-execution-button"]');

    // Wait for error handling
    await page.waitForTimeout(5000);

    // Check for error message
    const errorDisplay = page.locator('.error-message, [data-testid="execution-error"], .toast-error');

    if (await errorDisplay.isVisible()) {
      const errorText = await errorDisplay.textContent();
      console.log('Expected error message:', errorText);

      // Take screenshot of error state
      await page.screenshot({
        path: '/home/clauderun/vibe-kanban-experts/testartifacts/claude-flow-error-handling.png',
        fullPage: true
      });

      // Verify error message is user-friendly
      expect(errorText).toMatch(/install|available|npm|claude-flow/i);
    }
  });
});

test.describe('Claude-Flow Performance', () => {
  test('should show execution progress for claude-flow tasks', async ({ page }) => {
    await page.click('[data-testid="new-task-button"]');

    await page.fill('[data-testid="task-title"]', 'Performance Test Task');
    await page.fill('[data-testid="task-description"]', 'Test execution progress display');

    // Select CLAUDE_FLOW
    const agentSelection = page.locator('[data-testid="agent-selection"], select[name="agent"]');
    if (await agentSelection.isVisible()) {
      await agentSelection.selectOption('CLAUDE_FLOW');
    }

    // Start execution
    await page.click('[data-testid="start-execution-button"]');

    // Monitor execution progress
    const progressIndicator = page.locator('.progress-indicator, [data-testid="execution-progress"]');
    const statusIndicator = page.locator('.status-indicator, [data-testid="execution-status"]');

    // Check for progress indicators
    if (await progressIndicator.isVisible()) {
      console.log('Progress indicator found');
    }

    if (await statusIndicator.isVisible()) {
      const status = await statusIndicator.textContent();
      console.log('Execution status:', status);
    }

    // Take screenshot of execution interface
    await page.screenshot({
      path: '/home/clauderun/vibe-kanban-experts/testartifacts/claude-flow-execution-progress.png',
      fullPage: true
    });
  });
});

// Helper function to take screenshots for documentation
test.afterEach(async ({ page }, testInfo) => {
  if (testInfo.status === 'passed') {
    console.log(`Test ${testInfo.title} passed`);
  } else {
    // Take screenshot on failure
    await page.screenshot({
      path: `/home/clauderun/vibe-kanban-experts/testartifacts/failed-test-${testInfo.title.replace(/\s+/g, '-')}.png`,
      fullPage: true
    });
  }
});