import { describe, it, expect, vi, beforeEach } from 'vitest'
import { mount, flushPromises } from '@vue/test-utils'
import { createPinia, setActivePinia } from 'pinia'
import LoginView from '@/views/LoginView.vue'

const mockRouter = {
  push: vi.fn(),
  replace: vi.fn(),
}

const mockAuthStore = {
  checkSetupRequired: vi.fn().mockResolvedValue(false),
  login: vi.fn(),
  isAuthenticated: false,
}

vi.mock('vue-router', () => ({
  useRouter: () => mockRouter,
}))

vi.mock('@/stores/auth', () => ({
  useAuthStore: () => mockAuthStore,
}))

function mountView() {
  return mount(LoginView, {
    global: {
      plugins: [createPinia()],
      stubs: {
        Button: {
          template: '<button><slot /></button>',
          props: ['disabled'],
        },
      },
    },
  })
}

describe('LoginView', () => {
  beforeEach(() => {
    setActivePinia(createPinia())
    vi.clearAllMocks()
    mockAuthStore.checkSetupRequired.mockResolvedValue(false)
    mockAuthStore.login.mockReset()
  })

  it('renders login form', async () => {
    const wrapper = mountView()
    await flushPromises()

    expect(wrapper.find('input#username').exists()).toBe(true)
    expect(wrapper.find('input#password').exists()).toBe(true)
    expect(wrapper.find('button[type="submit"]').exists()).toBe(true)
  })

  it('shows error message on failed login', async () => {
    mockAuthStore.login.mockResolvedValue(false)
    const wrapper = mountView()
    await flushPromises()

    await wrapper.find('input#username').setValue('user')
    await wrapper.find('input#password').setValue('wrong')
    await wrapper.find('form').trigger('submit')
    await flushPromises()

    expect(wrapper.text()).toContain('Invalid username or password')
  })

  it('calls auth.login on form submit', async () => {
    mockAuthStore.login.mockResolvedValue(true)
    const wrapper = mountView()
    await flushPromises()

    await wrapper.find('input#username').setValue('admin')
    await wrapper.find('input#password').setValue('secret')
    await wrapper.find('form').trigger('submit')
    await flushPromises()

    expect(mockAuthStore.login).toHaveBeenCalledWith('admin', 'secret')
    expect(mockRouter.push).toHaveBeenCalledWith('/keys')
  })
})
