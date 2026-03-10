import { test, expect } from '@playwright/test'

test.describe('key settings (self-service)', () => {
  /** Ensure admin exists and return an admin JWT token. */
  async function ensureAdmin(request: any): Promise<string> {
    await request.post('/api/auth/setup', {
      data: { username: 'admin', password: 'testpassword123' },
    })
    const loginRes = await request.post('/api/auth/login', {
      data: { username: 'admin', password: 'testpassword123' },
    })
    const { token } = await loginRes.json()
    return token
  }

  /** Create admin + API key, login with key via UI. */
  async function loginWithApiKey(
    page: any,
    request: any,
  ): Promise<string> {
    const token = await ensureAdmin(request)

    const keyRes = await request.post('/api/keys', {
      headers: { Authorization: `Bearer ${token}` },
      data: { name: 'settings-test-key' },
    })
    const keyData = await keyRes.json()
    const apiKey = keyData.key

    // Login via UI with API key
    await page.goto('/login')
    await page.click('text=Sign in with API key instead')
    await page.fill('#apikey', apiKey)
    await page.click('button[type="submit"]')
    await expect(page).toHaveURL(/\/key-settings/)

    return apiKey
  }

  test('displays settings form with defaults', async ({ page, request }) => {
    // Ensure global settings are at a known state (other tests may change them)
    const adminToken = await ensureAdmin(request)
    await request.put('/api/admin/settings', {
      headers: {
        Authorization: `Bearer ${adminToken}`,
        'Content-Type': 'application/json',
      },
      data: { poster_source: 'tmdb' },
    })

    await loginWithApiKey(page, request)

    await expect(page.locator('h1')).toContainText('Poster Settings')
    await expect(page.locator('text=settings-test-key')).toBeVisible()

    // Settings form should be present with defaults
    const select = page.locator('select')
    await expect(select).toBeVisible()
    await expect(select).toHaveValue('tmdb')
  })

  test('auto-saves and shows confirmation', async ({ page, request }) => {
    await loginWithApiKey(page, request)

    // Change a setting to trigger auto-save
    await page.locator('select').selectOption('fanart')

    // Wait for auto-save confirmation
    await expect(page.locator('text=Saved')).toBeVisible({ timeout: 5000 })
  })

  test('fanart options appear when fanart is selected', async ({ page, request }) => {
    // Ensure global source is "tmdb" so fanart options start hidden
    const adminToken = await ensureAdmin(request)
    await request.put('/api/admin/settings', {
      headers: {
        Authorization: `Bearer ${adminToken}`,
        'Content-Type': 'application/json',
      },
      data: { poster_source: 'tmdb' },
    })

    await loginWithApiKey(page, request)

    // Language should not be visible initially
    await expect(page.locator('label:has-text("Language")')).not.toBeVisible()

    // Select fanart
    await page.locator('select').selectOption('fanart')

    // Now language and textless should appear
    await expect(page.locator('label:has-text("Language")')).toBeVisible()
    await expect(page.locator('label:has-text("Prefer textless")')).toBeVisible()
  })

  test('settings persist after auto-save and reload', async ({ page, request }) => {
    await loginWithApiKey(page, request)

    // Change to fanart
    await page.locator('select').selectOption('fanart')

    // Wait for auto-save confirmation
    await expect(page.locator('text=Saved')).toBeVisible({ timeout: 5000 })

    // Reload
    await page.reload()
    await expect(page.locator('h1')).toContainText('Poster Settings')

    // Settings should persist
    await expect(page.locator('select')).toHaveValue('fanart')
  })

  test('rating display section is visible', async ({ page, request }) => {
    await loginWithApiKey(page, request)

    await expect(page.locator('text=Rating Display')).toBeVisible()
    await expect(page.locator('text=Max ratings to show')).toBeVisible()
    await expect(page.locator('text=Rating order')).toBeVisible()
  })

  test('rating limit defaults to 3', async ({ page, request }) => {
    await loginWithApiKey(page, request)

    const limitInput = page.locator('input[type="number"]')
    await expect(limitInput).toBeVisible()
    await expect(limitInput).toHaveValue('3')
  })

  test('reset to defaults works', async ({ page, request }) => {
    // Ensure global settings are at a known state before testing reset.
    // Other tests (e.g. settings.spec.ts) may change global poster_source,
    // which would make the post-reset value unpredictable.
    const adminToken = await ensureAdmin(request)
    await request.put('/api/admin/settings', {
      headers: {
        Authorization: `Bearer ${adminToken}`,
        'Content-Type': 'application/json',
      },
      data: { poster_source: 'tmdb' },
    })

    await loginWithApiKey(page, request)

    // Global is now "tmdb", key has no overrides → should show "tmdb"
    await expect(page.locator('select')).toHaveValue('tmdb')

    // Change to fanart — auto-save triggers
    await page.locator('select').selectOption('fanart')
    await expect(page.locator('text=Saved')).toBeVisible({ timeout: 5000 })

    // Wait for "Using defaults" badge to disappear (confirms custom settings saved)
    await expect(page.locator('text=Using defaults')).not.toBeVisible()

    // Reset to defaults
    await page.locator('button:has-text("Reset to defaults")').click()

    // Should be back to global default ("tmdb")
    await expect(page.locator('text=Using defaults')).toBeVisible({ timeout: 10000 })
    await expect(page.locator('select')).toHaveValue('tmdb')
  })
})
