import { describe, it, expect, vi, beforeEach } from 'vitest'
import { createPinia, setActivePinia } from 'pinia'

const mockAuthStore = {
  token: 'test-token',
  refresh: vi.fn(),
  logout: vi.fn(),
}

vi.mock('@/stores/auth', () => ({
  useAuthStore: () => mockAuthStore,
}))

const mockRouter = {
  push: vi.fn(),
}

vi.mock('@/router', () => ({
  default: mockRouter,
}))

import { get, post, del } from '@/lib/api'

function makeFetchResponse(status: number, body: unknown = {}) {
  return {
    ok: status >= 200 && status < 300,
    status,
    json: () => Promise.resolve(body),
    text: () => Promise.resolve(JSON.stringify(body)),
  }
}

describe('api', () => {
  beforeEach(() => {
    setActivePinia(createPinia())
    vi.restoreAllMocks()
    mockAuthStore.token = 'test-token'
    mockAuthStore.refresh = vi.fn()
    mockAuthStore.logout = vi.fn()
    mockRouter.push = vi.fn()
  })

  it('get adds Authorization header when token exists', async () => {
    const fetchMock = vi.fn().mockResolvedValue(makeFetchResponse(200, { data: 'ok' }))
    vi.stubGlobal('fetch', fetchMock)

    await get('/api/test')

    expect(fetchMock).toHaveBeenCalledTimes(1)
    const [url, options] = fetchMock.mock.calls[0]
    expect(url).toBe('/api/test')
    expect(options.headers.get('Authorization')).toBe('Bearer test-token')
  })

  it('post sends JSON body', async () => {
    const fetchMock = vi.fn().mockResolvedValue(makeFetchResponse(200))
    vi.stubGlobal('fetch', fetchMock)

    await post('/api/items', { name: 'test' })

    const [, options] = fetchMock.mock.calls[0]
    expect(options.method).toBe('POST')
    expect(options.headers.get('Content-Type')).toBe('application/json')
    expect(options.body).toBe(JSON.stringify({ name: 'test' }))
  })

  it('del uses DELETE method', async () => {
    const fetchMock = vi.fn().mockResolvedValue(makeFetchResponse(200))
    vi.stubGlobal('fetch', fetchMock)

    await del('/api/items/1')

    const [, options] = fetchMock.mock.calls[0]
    expect(options.method).toBe('DELETE')
  })

  it('401 handling: attempts refresh, retries request on success', async () => {
    mockAuthStore.refresh.mockResolvedValue(true)
    mockAuthStore.token = 'refreshed-token'

    const fetchMock = vi
      .fn()
      .mockResolvedValueOnce(makeFetchResponse(401))
      .mockResolvedValueOnce(makeFetchResponse(200, { data: 'ok' }))
    vi.stubGlobal('fetch', fetchMock)

    await get('/api/protected')

    expect(mockAuthStore.refresh).toHaveBeenCalled()
    expect(fetchMock).toHaveBeenCalledTimes(2)
  })

  it('401 + failed refresh: calls logout', async () => {
    mockAuthStore.refresh.mockResolvedValue(false)

    const fetchMock = vi.fn().mockResolvedValue(makeFetchResponse(401))
    vi.stubGlobal('fetch', fetchMock)

    await get('/api/protected')

    expect(mockAuthStore.refresh).toHaveBeenCalled()
    expect(mockAuthStore.logout).toHaveBeenCalled()
    expect(mockRouter.push).toHaveBeenCalledWith({ name: 'login' })
  })
})
