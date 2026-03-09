import { describe, it, expect, vi, beforeEach } from 'vitest'
import { mount, flushPromises } from '@vue/test-utils'
import { createPinia, setActivePinia } from 'pinia'
import ApiKeysView from '@/views/ApiKeysView.vue'

const mockRouter = {
  push: vi.fn(),
}

const mockAuthStore = {
  logout: vi.fn(),
}

const mockApi = {
  get: vi.fn(),
  post: vi.fn(),
  del: vi.fn(),
}

vi.mock('vue-router', () => ({
  useRouter: () => mockRouter,
}))

vi.mock('@/stores/auth', () => ({
  useAuthStore: () => mockAuthStore,
}))

vi.mock('@/lib/api', () => ({
  get: (...args: unknown[]) => mockApi.get(...args),
  post: (...args: unknown[]) => mockApi.post(...args),
  del: (...args: unknown[]) => mockApi.del(...args),
}))

const sampleKeys = [
  {
    id: 1,
    name: 'jellyfin-prod',
    key_prefix: 'opdb_abc',
    created_at: '2026-01-01',
    last_used_at: null,
  },
  {
    id: 2,
    name: 'plex-dev',
    key_prefix: 'opdb_xyz',
    created_at: '2026-02-01',
    last_used_at: '2026-03-01',
  },
]

function mountView() {
  return mount(ApiKeysView, {
    global: {
      plugins: [createPinia()],
      stubs: {
        Button: {
          template: '<button @click="$emit(\'click\')"><slot /></button>',
          props: ['disabled', 'variant', 'size'],
        },
      },
    },
  })
}

describe('ApiKeysView', () => {
  beforeEach(() => {
    setActivePinia(createPinia())
    vi.clearAllMocks()
    mockApi.get.mockResolvedValue({
      ok: true,
      json: () => Promise.resolve([]),
    })
  })

  it('renders key list from mocked API response', async () => {
    mockApi.get.mockResolvedValue({
      ok: true,
      json: () => Promise.resolve(sampleKeys),
    })

    const wrapper = mountView()
    await flushPromises()

    expect(wrapper.text()).toContain('jellyfin-prod')
    expect(wrapper.text()).toContain('plex-dev')
    expect(wrapper.text()).toContain('opdb_abc')
    expect(wrapper.text()).toContain('opdb_xyz')
  })

  it('shows "No API keys yet." when empty', async () => {
    mockApi.get.mockResolvedValue({
      ok: true,
      json: () => Promise.resolve([]),
    })

    const wrapper = mountView()
    await flushPromises()

    expect(wrapper.text()).toContain('No API keys yet.')
  })

  it('logout button calls auth.logout', async () => {
    const wrapper = mountView()
    await flushPromises()

    const buttons = wrapper.findAll('button')
    const logoutButton = buttons.find((b) => b.text().includes('Sign out'))
    expect(logoutButton).toBeDefined()

    await logoutButton!.trigger('click')

    expect(mockAuthStore.logout).toHaveBeenCalled()
    expect(mockRouter.push).toHaveBeenCalledWith('/login')
  })
})
