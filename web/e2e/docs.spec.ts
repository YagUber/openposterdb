import { test, expect } from '@playwright/test'

test.describe('docs page', () => {
  test.beforeEach(async ({ request }) => {
    await request.post('/api/auth/setup', {
      data: { username: 'admin', password: 'testpassword123' },
    })
  })

  test('loads docs page', async ({ page }) => {
    await page.goto('/docs')
    await expect(page.locator('text=API Reference')).toBeVisible()
  })

  test('back link navigates home', async ({ page }) => {
    await page.goto('/docs')
    const backLink = page.locator('a:has-text("OpenPosterDB")')
    await expect(backLink).toBeVisible()
    await backLink.click()
    await expect(page).toHaveURL('/')
  })
})
