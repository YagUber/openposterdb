import { describe, it, expect, vi, beforeEach } from 'vitest'
import { mount, flushPromises } from '@vue/test-utils'
import { createPinia, setActivePinia } from 'pinia'
import { QueryClient, VueQueryPlugin } from '@tanstack/vue-query'
import { createRouter, createMemoryHistory } from 'vue-router'
import ImageListView from '@/components/ImageListView.vue'

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

function makeMocks() {
  return {
    listFn: vi.fn(),
    imageFn: vi.fn(),
    fetchFn: vi.fn(),
  }
}

function mountView(mocks: ReturnType<typeof makeMocks>, kind: 'poster' | 'logo' | 'backdrop' = 'poster') {
  const queryClient = new QueryClient({
    defaultOptions: { queries: { retry: false } },
  })
  const router = createRouter({
    history: createMemoryHistory(),
    routes: [{ path: '/', component: { template: '<div />' } }],
  })
  return mount(ImageListView, {
    props: {
      kind,
      listFn: mocks.listFn,
      imageFn: mocks.imageFn,
      fetchFn: mocks.fetchFn,
    },
    global: {
      plugins: [createPinia(), router, [VueQueryPlugin, { queryClient }]],
      stubs: {
        Button: {
          template: '<button @click="$emit(\'click\')" :disabled="disabled"><slot /></button>',
          props: ['disabled', 'variant', 'size', 'type'],
        },
        Skeleton: { template: '<div data-testid="skeleton" />' },
        Table: { template: '<table><slot /></table>' },
        TableHeader: { template: '<thead><slot /></thead>' },
        TableBody: { template: '<tbody><slot /></tbody>' },
        TableRow: { template: '<tr><slot /></tr>' },
        TableHead: { template: '<th><slot /></th>' },
        TableCell: { template: '<td><slot /></td>' },
        RefreshButton: {
          template: '<button @click="$emit(\'refresh\')">Refresh</button>',
          props: ['fetching'],
          emits: ['refresh'],
        },
        Dialog: { template: '<div v-if="open"><slot /></div>', props: ['open'] },
        DialogContent: { template: '<div><slot /></div>' },
        DialogHeader: { template: '<div><slot /></div>' },
        DialogTitle: { template: '<div><slot /></div>' },
        Input: {
          template: '<input :value="modelValue" @input="$emit(\'update:modelValue\', $event.target.value)" />',
          props: ['modelValue'],
          emits: ['update:modelValue'],
        },
        Download: { template: '<span />' },
        Loader2: { template: '<span />' },
        Eye: { template: '<span />' },
      },
    },
  })
}

describe('ImageListView', () => {
  beforeEach(() => {
    setActivePinia(createPinia())
    vi.clearAllMocks()
  })

  it('shows skeletons while loading', () => {
    const mocks = makeMocks()
    mocks.listFn.mockReturnValue(new Promise(() => {}))
    const wrapper = mountView(mocks)
    expect(wrapper.findAll('[data-testid="skeleton"]').length).toBeGreaterThan(0)
  })

  it('renders list with parsed cache keys', async () => {
    const mocks = makeMocks()
    mocks.listFn.mockResolvedValue({
      ok: true,
      json: () => Promise.resolve(sampleResponse),
    })

    const wrapper = mountView(mocks)
    await flushPromises()

    expect(wrapper.text()).toContain('imdb')
    expect(wrapper.text()).toContain('tt0111161')
    expect(wrapper.text()).toContain('tmdb')
    expect(wrapper.text()).toContain('550')
    expect(wrapper.text()).toContain('1994-09-23')
  })

  it('shows empty state when no items', async () => {
    const mocks = makeMocks()
    mocks.listFn.mockResolvedValue({
      ok: true,
      json: () => Promise.resolve({ items: [], total: 0, page: 1, page_size: 50 }),
    })

    const wrapper = mountView(mocks)
    await flushPromises()

    expect(wrapper.text()).toContain('No posters cached yet.')
  })

  it('shows total count and pagination info', async () => {
    const mocks = makeMocks()
    mocks.listFn.mockResolvedValue({
      ok: true,
      json: () => Promise.resolve(sampleResponse),
    })

    const wrapper = mountView(mocks)
    await flushPromises()

    expect(wrapper.text()).toContain('2 posters total')
    expect(wrapper.text()).toContain('Page 1 of 1')
  })

  it('has a refresh button', async () => {
    const mocks = makeMocks()
    mocks.listFn.mockResolvedValue({
      ok: true,
      json: () => Promise.resolve(sampleResponse),
    })

    const wrapper = mountView(mocks)
    await flushPromises()

    const refreshButton = wrapper.findAll('button').find((b) => b.text().includes('Refresh'))
    expect(refreshButton).toBeDefined()
  })

  it('has a fetch button', async () => {
    const mocks = makeMocks()
    mocks.listFn.mockResolvedValue({
      ok: true,
      json: () => Promise.resolve(sampleResponse),
    })

    const wrapper = mountView(mocks)
    await flushPromises()

    const fetchButton = wrapper.findAll('button').find((b) => b.text().includes('Fetch'))
    expect(fetchButton).toBeDefined()
  })

  it('opens fetch modal when fetch button is clicked', async () => {
    const mocks = makeMocks()
    mocks.listFn.mockResolvedValue({
      ok: true,
      json: () => Promise.resolve(sampleResponse),
    })

    const wrapper = mountView(mocks)
    await flushPromises()

    // Modal content should not be visible initially
    expect(wrapper.text()).not.toContain('Fetch Poster')

    const fetchButton = wrapper.findAll('button').find((b) => b.text().includes('Fetch'))
    await fetchButton!.trigger('click')
    await flushPromises()

    expect(wrapper.text()).toContain('Fetch Poster')
    expect(wrapper.text()).toContain('ID Type')
    expect(wrapper.text()).toContain('ID Value')
  })

  it('calls fetchFn on form submit', async () => {
    const mocks = makeMocks()
    mocks.listFn.mockResolvedValue({
      ok: true,
      json: () => Promise.resolve(sampleResponse),
    })
    mocks.fetchFn.mockResolvedValue({
      ok: true,
      blob: () => Promise.resolve(new Blob()),
    })
    mocks.imageFn.mockResolvedValue({
      ok: true,
      blob: () => Promise.resolve(new Blob()),
    })

    const wrapper = mountView(mocks)
    await flushPromises()

    // Open modal
    const fetchButton = wrapper.findAll('button').find((b) => b.text().includes('Fetch'))
    await fetchButton!.trigger('click')
    await flushPromises()

    // Fill in ID value
    const input = wrapper.find('input')
    await input.setValue('tt0111161')
    await flushPromises()

    // Submit form
    const form = wrapper.find('form')
    await form.trigger('submit')
    await flushPromises()

    expect(mocks.fetchFn).toHaveBeenCalledWith('imdb', 'tt0111161')
  })
})
