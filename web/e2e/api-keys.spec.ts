import { test, expect } from '@playwright/test'

test.describe('api keys', () => {
  test.beforeEach(async ({ page, request }) => {
    // Ensure admin exists and login
    await request.post('/api/auth/setup', {
      data: { username: 'admin', password: 'testpassword123' },
    })

    await page.goto('/login')
    await page.fill('#username', 'admin')
    await page.fill('#password', 'testpassword123')
    await page.click('button[type="submit"]')
    await expect(page).toHaveURL(/\/$/)

    // Navigate to API Keys page
    await page.click('text=API Keys')
    await expect(page).toHaveURL(/\/keys/)
  })

  test('create a new API key and see its value', async ({ page }) => {
    await page.fill('input[placeholder*="Key name"]', 'test-key')
    await page.click('button:has-text("Create")')

    // Key value banner should appear
    await expect(page.locator('text=Copy your API key now')).toBeVisible()
    await expect(page.locator('code')).toBeVisible()
  })

  test('dismiss key banner', async ({ page }) => {
    await page.fill('input[placeholder*="Key name"]', 'dismiss-test')
    await page.click('button:has-text("Create")')

    await expect(page.locator('text=Copy your API key now')).toBeVisible()
    await page.click('button:has-text("Dismiss")')
    await expect(page.locator('text=Copy your API key now')).not.toBeVisible()
  })

  test('key appears in list with correct name', async ({ page }) => {
    await page.fill('input[placeholder*="Key name"]', 'my-prod-key')
    await page.click('button:has-text("Create")')

    // Key should appear in the list
    await expect(page.locator('text=my-prod-key')).toBeVisible()
  })

  test('refresh button is visible and clickable', async ({ page }) => {
    const refreshButton = page.locator('button:has-text("Refresh")')
    await expect(refreshButton).toBeVisible()

    await refreshButton.click()

    // Button should still be present after refresh completes
    await expect(refreshButton).toBeVisible()
  })

  test('delete a key removes it from list', async ({ page }) => {
    // Create a key
    await page.fill('input[placeholder*="Key name"]', 'to-delete')
    await page.click('button:has-text("Create")')
    await expect(page.locator('text=to-delete')).toBeVisible()

    // Accept the confirm dialog
    page.on('dialog', (dialog) => dialog.accept())

    // Click delete on the key
    await page.locator('text=to-delete').locator('..').locator('..').locator('button:has-text("Delete")').click()

    // Key should be gone
    await expect(page.locator('text=to-delete')).not.toBeVisible()
  })
})
