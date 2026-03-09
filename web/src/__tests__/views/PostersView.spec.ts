import { describe, it, expect, vi, beforeEach } from 'vitest'
import { mount, flushPromises } from '@vue/test-utils'
import { createPinia, setActivePinia } from 'pinia'
import { QueryClient, VueQueryPlugin } from '@tanstack/vue-query'
import { createRouter, createMemoryHistory } from 'vue-router'
import PostersView from '@/views/PostersView.vue'

const mockAdminApi = vi.hoisted(() => ({
  getPosters: vi.fn(),
  getPosterImage: vi.fn(),
}))

vi.mock('@/lib/api', () => ({
  adminApi: mockAdminApi,
}))

const sampleResponse = {
  items: [
    {
      cache_key: 'imdb/tt0111161',
      release_date: '1994-09-23',
      created_at: 1710000000,
      updated_at: 1710100000,
    },
    {
      cache_key: 'tmdb/550',
      release_date: '1999-10-15',
      created_at: 1710000000,
      updated_at: 1710100000,
    },
  ],
  total: 2,
  page: 1,
  page_size: 50,
}

function mountView() {
  const queryClient = new QueryClient({
    defaultOptions: { queries: { retry: false } },
  })
  const router = createRouter({
    history: createMemoryHistory(),
    routes: [{ path: '/', component: PostersView }],
  })
  return mount(PostersView, {
    global: {
      plugins: [createPinia(), router, [VueQueryPlugin, { queryClient }]],
      stubs: {
        Button: {
          template: '<button @click="$emit(\'click\')" :disabled="disabled"><slot /></button>',
          props: ['disabled', 'variant', 'size'],
        },
        Skeleton: { template: '<div data-testid="skeleton" />' },
        Table: { template: '<table><slot /></table>' },
        TableHeader: { template: '<thead><slot /></thead>' },
        TableBody: { template: '<tbody><slot /></tbody>' },
        TableRow: { template: '<tr><slot /></tr>' },
        TableHead: { template: '<th><slot /></th>' },
        TableCell: { template: '<td><slot /></td>' },
        RefreshCw: { template: '<span />' },
      },
    },
  })
}

describe('PostersView', () => {
  beforeEach(() => {
    setActivePinia(createPinia())
    vi.clearAllMocks()
  })

  it('shows skeletons while loading', () => {
    mockAdminApi.getPosters.mockReturnValue(new Promise(() => {}))
    const wrapper = mountView()
    expect(wrapper.findAll('[data-testid="skeleton"]').length).toBeGreaterThan(0)
  })

  it('renders poster list with parsed cache keys', async () => {
    mockAdminApi.getPosters.mockResolvedValue({
      ok: true,
      json: () => Promise.resolve(sampleResponse),
    })

    const wrapper = mountView()
    await flushPromises()

    expect(wrapper.text()).toContain('imdb')
    expect(wrapper.text()).toContain('tt0111161')
    expect(wrapper.text()).toContain('tmdb')
    expect(wrapper.text()).toContain('550')
    expect(wrapper.text()).toContain('1994-09-23')
  })

  it('shows empty state when no posters', async () => {
    mockAdminApi.getPosters.mockResolvedValue({
      ok: true,
      json: () => Promise.resolve({ items: [], total: 0, page: 1, page_size: 50 }),
    })

    const wrapper = mountView()
    await flushPromises()

    expect(wrapper.text()).toContain('No posters cached yet.')
  })

  it('shows total count and pagination info', async () => {
    mockAdminApi.getPosters.mockResolvedValue({
      ok: true,
      json: () => Promise.resolve(sampleResponse),
    })

    const wrapper = mountView()
    await flushPromises()

    expect(wrapper.text()).toContain('2 posters total')
    expect(wrapper.text()).toContain('Page 1 of 1')
  })

  it('has a refresh button', async () => {
    mockAdminApi.getPosters.mockResolvedValue({
      ok: true,
      json: () => Promise.resolve(sampleResponse),
    })

    const wrapper = mountView()
    await flushPromises()

    const refreshButton = wrapper.findAll('button').find((b) => b.text().includes('Refresh'))
    expect(refreshButton).toBeDefined()
  })

  it('refresh button triggers refetch', async () => {
    mockAdminApi.getPosters.mockResolvedValue({
      ok: true,
      json: () => Promise.resolve(sampleResponse),
    })

    const wrapper = mountView()
    await flushPromises()

    mockAdminApi.getPosters.mockClear()
    mockAdminApi.getPosters.mockResolvedValue({
      ok: true,
      json: () => Promise.resolve({ ...sampleResponse, total: 10 }),
    })

    const refreshButton = wrapper.findAll('button').find((b) => b.text().includes('Refresh'))
    await refreshButton!.trigger('click')
    await flushPromises()

    expect(mockAdminApi.getPosters).toHaveBeenCalled()
    expect(wrapper.text()).toContain('10 posters total')
  })
})
