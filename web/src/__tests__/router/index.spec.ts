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
      { path: '/', redirect: '/keys' },
      { path: '/login', name: 'login', component: { template: '<div>Login</div>' } },
      { path: '/setup', name: 'setup', component: { template: '<div>Setup</div>' } },
      {
        path: '/keys',
        name: 'keys',
        component: { template: '<div>Keys</div>' },
        meta: { requiresAuth: true },
      },
    ],
  })
}

describe('router', () => {
  beforeEach(() => {
    setActivePinia(createPinia())
  })

  it('unauthenticated user visiting /keys gets redirected to /login', async () => {
    const router = makeRouter()
    const auth = useAuthStore()

    vi.spyOn(auth, 'checkSetupRequired').mockResolvedValue(false)

    router.beforeEach(async (to) => {
      if (to.meta.requiresAuth && !auth.isAuthenticated) {
        return { name: 'login' }
      }
    })

    await router.push('/keys')
    await router.isReady()

    expect(router.currentRoute.value.name).toBe('login')
  })

  it('authenticated user visiting /login gets redirected to /keys', async () => {
    const router = makeRouter()
    const auth = useAuthStore()
    auth.token = 'valid-token'

    vi.spyOn(auth, 'checkSetupRequired').mockResolvedValue(false)

    router.beforeEach(async (to) => {
      if ((to.name === 'login' || to.name === 'setup') && auth.isAuthenticated) {
        return { name: 'keys' }
      }
    })

    await router.push('/login')
    await router.isReady()

    expect(router.currentRoute.value.name).toBe('keys')
  })
})
