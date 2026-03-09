import { test, expect } from '@playwright/test'

test.describe.serial('setup flow', () => {
  test('fresh app redirects to /setup', async ({ page }) => {
    await page.goto('/')
    await expect(page).toHaveURL(/\/setup/)
  })

  test('create admin account and redirect to /keys', async ({ page }) => {
    await page.goto('/setup')

    await page.fill('#username', 'admin')
    await page.fill('#password', 'testpassword123')
    await page.fill('#confirm-password', 'testpassword123')
    await page.click('button[type="submit"]')

    await expect(page).toHaveURL(/\/keys/)
  })

  test('revisiting /setup after account created redirects to /login', async ({ page }) => {
    await page.goto('/setup')
    await expect(page).toHaveURL(/\/login/)
  })
})
