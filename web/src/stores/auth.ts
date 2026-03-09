import { defineStore } from 'pinia'
import { ref, computed } from 'vue'
import { authApi } from '@/lib/auth-api'

export const useAuthStore = defineStore('auth', () => {
  const token = ref<string | null>(localStorage.getItem('token'))
  const isAuthenticated = computed(() => !!token.value)
  const setupRequired = ref<boolean | null>(null)

  function setToken(t: string) {
    token.value = t
    localStorage.setItem('token', t)
  }

  function clearToken() {
    token.value = null
    localStorage.removeItem('token')
  }

  async function checkSetupRequired(): Promise<boolean> {
    if (setupRequired.value !== null) return setupRequired.value
    const res = await authApi.status()
    if (!res.ok) throw new Error(`status check failed: ${res.status}`)
    const data = await res.json()
    setupRequired.value = data.setup_required
    return data.setup_required
  }

  async function setup(username: string, password: string): Promise<boolean> {
    const res = await authApi.setup(username, password)
    if (!res.ok) return false
    const data = await res.json()
    setToken(data.token)
    setupRequired.value = false
    return true
  }

  async function login(username: string, password: string): Promise<boolean> {
    const res = await authApi.login(username, password)
    if (!res.ok) return false
    const data = await res.json()
    setToken(data.token)
    return true
  }

  async function refresh(): Promise<boolean> {
    const res = await authApi.refresh()
    if (!res.ok) return false
    const data = await res.json()
    setToken(data.token)
    return true
  }

  function logout() {
    clearToken()
    setupRequired.value = null
  }

  return { token, isAuthenticated, setupRequired, checkSetupRequired, setup, login, refresh, logout }
})
