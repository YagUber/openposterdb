import { test, expect } from '@playwright/test'

test.describe('not found page', () => {
  test.beforeEach(async ({ request }) => {
    await request.post('/api/auth/setup', {
      data: { username: 'admin', password: 'testpassword123' },
    })
  })

  test('shows 404 page for unknown route', async ({ page }) => {
    await page.goto('/nonexistent')
    await expect(page.locator('h1')).toContainText('404')
  })

  test('Go home navigates to /', async ({ page }) => {
    await page.goto('/nonexistent')
    await expect(page.locator('h1')).toContainText('404')

    const goHomeButton = page.locator('a:has-text("Go home")')
    await expect(goHomeButton).toBeVisible()
    await goHomeButton.click()
    await expect(page).toHaveURL('/')
  })
})
