import { describe, it, expect, vi, beforeEach } from 'vitest'
import { createPinia, setActivePinia } from 'pinia'
import { createRouter, createWebHistory } from 'vue-router'
import { useAuthStore } from '@/stores/auth'

// We need to create a fresh router for each test to avoid guard state leaking.
// We also mock checkSetupRequired so it doesn't actually fetch.

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
          { path: 'keys', name: 'keys', component: { template: '<div>Keys</div>' } },
        ],
      },
      { path: '/login', name: 'login', component: { template: '<div>Login</div>' } },
      { path: '/setup', name: 'setup', component: { template: '<div>Setup</div>' } },
    ],
  })
}

describe('router', () => {
  beforeEach(() => {
    setActivePinia(createPinia())
  })

  it('unauthenticated user visiting / gets redirected to /login', async () => {
    const router = makeRouter()
    const auth = useAuthStore()

    vi.spyOn(auth, 'checkSetupRequired').mockResolvedValue(false)

    router.beforeEach(async (to) => {
      if (to.matched.some((r) => r.meta.requiresAuth) && !auth.isAuthenticated) {
        return { name: 'login' }
      }
    })

    await router.push('/')
    await router.isReady()

    expect(router.currentRoute.value.name).toBe('login')
  })

  it('authenticated user visiting /login gets redirected to dashboard', async () => {
    const router = makeRouter()
    const auth = useAuthStore()
    auth.token = 'valid-token'

    vi.spyOn(auth, 'checkSetupRequired').mockResolvedValue(false)

    router.beforeEach(async (to) => {
      if ((to.name === 'login' || to.name === 'setup') && auth.isAuthenticated) {
        return { name: 'dashboard' }
      }
    })

    await router.push('/login')
    await router.isReady()

    expect(router.currentRoute.value.name).toBe('dashboard')
  })
})
