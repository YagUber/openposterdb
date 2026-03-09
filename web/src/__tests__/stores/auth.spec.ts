import { describe, it, expect, vi, beforeEach } from 'vitest'
import { createPinia, setActivePinia } from 'pinia'
import { useAuthStore } from '@/stores/auth'

const localStorageMock: Record<string, string> = {}
const localStorageStub = {
  getItem: vi.fn((key: string) => localStorageMock[key] ?? null),
  setItem: vi.fn((key: string, value: string) => {
    localStorageMock[key] = value
  }),
  removeItem: vi.fn((key: string) => {
    delete localStorageMock[key]
  }),
}

vi.stubGlobal('localStorage', localStorageStub)

function mockFetchSuccess(data: Record<string, unknown>, ok = true) {
  return vi.fn().mockResolvedValue({
    ok,
    status: ok ? 200 : 401,
    json: () => Promise.resolve(data),
  })
}

describe('auth store', () => {
  beforeEach(() => {
    setActivePinia(createPinia())
    Object.keys(localStorageMock).forEach((k) => delete localStorageMock[k])
    vi.restoreAllMocks()
    vi.stubGlobal('localStorage', localStorageStub)
  })

  it('token is set via login and stored in localStorage', async () => {
    vi.stubGlobal('fetch', mockFetchSuccess({ token: 'abc123' }))
    const auth = useAuthStore()
    await auth.login('user', 'pass')
    expect(auth.token).toBe('abc123')
    expect(localStorageStub.setItem).toHaveBeenCalledWith('token', 'abc123')
  })

  it('logout clears token from localStorage and sets null', async () => {
    vi.stubGlobal('fetch', mockFetchSuccess({ token: 'abc123' }))
    const auth = useAuthStore()
    await auth.login('user', 'pass')
    auth.logout()
    expect(auth.token).toBeNull()
    expect(localStorageStub.removeItem).toHaveBeenCalledWith('token')
  })

  it('isAuthenticated is true when token set', async () => {
    vi.stubGlobal('fetch', mockFetchSuccess({ token: 'abc123' }))
    const auth = useAuthStore()
    await auth.login('user', 'pass')
    expect(auth.isAuthenticated).toBe(true)
  })

  it('isAuthenticated is false when token is null', () => {
    const auth = useAuthStore()
    expect(auth.isAuthenticated).toBe(false)
  })

  it('login returns true on success', async () => {
    const fetchMock = mockFetchSuccess({ token: 'new-token' })
    vi.stubGlobal('fetch', fetchMock)
    const auth = useAuthStore()

    const result = await auth.login('user', 'pass')

    expect(result).toBe(true)
    expect(auth.token).toBe('new-token')
    expect(fetchMock).toHaveBeenCalledWith(
      expect.stringContaining('/api/auth/login'),
      expect.objectContaining({ method: 'POST' }),
    )
  })

  it('login returns false on failure', async () => {
    const fetchMock = mockFetchSuccess({}, false)
    vi.stubGlobal('fetch', fetchMock)
    const auth = useAuthStore()

    const result = await auth.login('user', 'wrong')

    expect(result).toBe(false)
    expect(auth.token).toBeNull()
  })

  it('setup sets token and setupRequired=false', async () => {
    const fetchMock = mockFetchSuccess({ token: 'setup-token' })
    vi.stubGlobal('fetch', fetchMock)
    const auth = useAuthStore()

    const result = await auth.setup('admin', 'password123')

    expect(result).toBe(true)
    expect(auth.token).toBe('setup-token')
    expect(auth.setupRequired).toBe(false)
  })

  it('refresh updates token', async () => {
    const fetchMock = mockFetchSuccess({ token: 'refreshed-token' })
    vi.stubGlobal('fetch', fetchMock)
    const auth = useAuthStore()

    const result = await auth.refresh()

    expect(result).toBe(true)
    expect(auth.token).toBe('refreshed-token')
  })

  it('logout clears token and sets setupRequired to null', async () => {
    vi.stubGlobal('fetch', mockFetchSuccess({ token: 'abc' }))
    const auth = useAuthStore()
    await auth.login('user', 'pass')
    auth.logout()
    expect(auth.token).toBeNull()
    expect(auth.setupRequired).toBeNull()
  })

  it('checkSetupRequired caches result on second call', async () => {
    const fetchMock = mockFetchSuccess({ setup_required: true })
    vi.stubGlobal('fetch', fetchMock)
    const auth = useAuthStore()

    const first = await auth.checkSetupRequired()
    const second = await auth.checkSetupRequired()

    expect(first).toBe(true)
    expect(second).toBe(true)
    expect(fetchMock).toHaveBeenCalledTimes(1)
  })
})
