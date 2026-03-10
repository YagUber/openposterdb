import { describe, it, expect, vi, beforeEach } from 'vitest'
import { createPinia, setActivePinia } from 'pinia'
import { createRouter, createWebHistory } from 'vue-router'
import { useAuthStore } from '@/stores/auth'

vi.stubGlobal('sessionStorage', {
  getItem: vi.fn(() => null),
  setItem: vi.fn(),
  removeItem: vi.fn(),
})

function makeRouter() {
  return createRouter({
    history: createWebHistory(),
    routes: [
      {
        path: '/',
        component: { template: '<router-view />' },
        meta: { requiresAuth: true },
        children: [
          { path: '', name: 'dashboard', component: { template: '<div>Dashboard</div>' } },
          { path: 'posters', name: 'posters', component: { template: '<div>Posters</div>' } },
          { path: 'logos', name: 'logos', component: { template: '<div>Logos</div>' } },
          { path: 'backdrops', name: 'backdrops', component: { template: '<div>Backdrops</div>' } },
          { path: 'keys', name: 'keys', component: { template: '<div>Keys</div>' } },
        ],
      },
      {
        path: '/key-settings',
        name: 'key-settings',
        component: { template: '<div>Key Settings</div>' },
        meta: { requiresApiKey: true },
      },
      { path: '/login', name: 'login', component: { template: '<div>Login</div>' } },
      { path: '/setup', name: 'setup', component: { template: '<div>Setup</div>' } },
    ],
  })
}

function addGuards(router: ReturnType<typeof createRouter>) {
  const auth = useAuthStore()
  router.beforeEach(async (to) => {
    // Admin routes require admin session
    if (to.matched.some((r) => r.meta.requiresAuth)) {
      if (!auth.isAdminSession) {
        if (auth.isApiKeySession) {
          return { name: 'key-settings' }
        }
        return { name: 'login' }
      }
    }

    // API key routes require API key session
    if (to.matched.some((r) => r.meta.requiresApiKey) && !auth.isApiKeySession) {
      return { name: 'login' }
    }

    // Redirect away from login/setup if already authenticated
    if (to.name === 'login' || to.name === 'setup') {
      if (auth.isAdminSession) {
        return { name: 'dashboard' }
      }
      if (auth.isApiKeySession) {
        return { name: 'key-settings' }
      }
    }
  })
}

describe('router', () => {
  beforeEach(() => {
    setActivePinia(createPinia())
  })

  it('unauthenticated user visiting / gets redirected to /login', async () => {
    const router = makeRouter()
    addGuards(router)

    await router.push('/')
    await router.isReady()

    expect(router.currentRoute.value.name).toBe('login')
  })

  it('admin user visiting /login gets redirected to dashboard', async () => {
    const router = makeRouter()
    const auth = useAuthStore()
    auth.token = 'valid-token'
    addGuards(router)

    await router.push('/login')
    await router.isReady()

    expect(router.currentRoute.value.name).toBe('dashboard')
  })

  it('API key user visiting / gets redirected to key-settings', async () => {
    const router = makeRouter()
    const auth = useAuthStore()
    auth.apiKeyToken = 'jwt-token'
    addGuards(router)

    await router.push('/')
    await router.isReady()

    expect(router.currentRoute.value.name).toBe('key-settings')
  })

  it('API key user visiting /login gets redirected to key-settings', async () => {
    const router = makeRouter()
    const auth = useAuthStore()
    auth.apiKeyToken = 'jwt-token'
    addGuards(router)

    await router.push('/login')
    await router.isReady()

    expect(router.currentRoute.value.name).toBe('key-settings')
  })

  it('unauthenticated user visiting /key-settings gets redirected to /login', async () => {
    const router = makeRouter()
    addGuards(router)

    await router.push('/key-settings')
    await router.isReady()

    expect(router.currentRoute.value.name).toBe('login')
  })

  it('API key user can access /key-settings', async () => {
    const router = makeRouter()
    const auth = useAuthStore()
    auth.apiKeyToken = 'jwt-token'
    addGuards(router)

    await router.push('/key-settings')
    await router.isReady()

    expect(router.currentRoute.value.name).toBe('key-settings')
  })

  it('admin user can access admin routes', async () => {
    const router = makeRouter()
    const auth = useAuthStore()
    auth.token = 'valid-token'
    addGuards(router)

    await router.push('/')
    await router.isReady()

    expect(router.currentRoute.value.name).toBe('dashboard')
  })
})
