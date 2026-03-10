import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest'
import { mount, flushPromises } from '@vue/test-utils'
import { createPinia, setActivePinia } from 'pinia'
import PosterSettingsForm from '@/components/PosterSettingsForm.vue'
import type { PosterSettings } from '@/components/PosterSettingsForm.vue'

vi.mock('@/lib/api', () => ({
  BASE_URL: 'http://test-api',
}))

const defaultSettings: PosterSettings = {
  poster_source: 'tmdb',
  fanart_lang: 'en',
  fanart_textless: false,
  fanart_available: true,
  ratings_limit: 3,
  ratings_order: 'mal,imdb,lb,rt,rta,mc,tmdb,trakt',
}

function mountForm(overrides: Partial<PosterSettings> = {}) {
  const settings = { ...defaultSettings, ...overrides }
  return mount(PosterSettingsForm, {
    props: {
      settings,
      loadSettings: vi.fn().mockResolvedValue(settings),
      saveSettings: vi.fn().mockResolvedValue(null),
    },
    global: {
      plugins: [createPinia()],
      stubs: {
        Button: {
          template: '<button :disabled="disabled" @click="$emit(\'click\')"><slot /></button>',
          props: ['disabled', 'variant', 'size'],
        },
      },
    },
  })
}

describe('PosterSettingsForm', () => {
  beforeEach(() => {
    setActivePinia(createPinia())
    vi.useFakeTimers()
  })

  afterEach(() => {
    vi.useRealTimers()
  })

  it('renders preview section', () => {
    const wrapper = mountForm()
    expect(wrapper.text()).toContain('Preview')
    expect(wrapper.find('img[alt="Poster preview"]').exists()).toBe(true)
  })

  it('sets initial preview src on mount', () => {
    const wrapper = mountForm()
    const img = wrapper.find('img[alt="Poster preview"]')
    const src = img.attributes('src')!
    expect(src).toContain('http://test-api/api/preview/poster')
    expect(src).toContain('ratings_limit=3')
    expect(src).toContain('ratings_order=mal')
  })

  it('preview URL includes current ratings_limit and ratings_order', () => {
    const wrapper = mountForm({
      ratings_limit: 5,
      ratings_order: 'imdb,rt,tmdb',
    })
    const src = wrapper.find('img[alt="Poster preview"]').attributes('src')!
    expect(src).toContain('ratings_limit=5')
    expect(src).toContain('ratings_order=imdb')
    expect(src).toContain('rt')
    expect(src).toContain('tmdb')
  })

  it('updates preview URL after debounce when ratings_limit changes', async () => {
    const wrapper = mountForm()
    const initialSrc = wrapper.find('img[alt="Poster preview"]').attributes('src')!

    // Change the limit via the native input (not stubbed)
    const limitInput = wrapper.find('input[type="number"]')
    await limitInput.setValue(5)
    await flushPromises()

    // Before debounce fires, src should still be the old one
    expect(wrapper.find('img[alt="Poster preview"]').attributes('src')).toBe(initialSrc)

    // Advance past debounce timer
    vi.advanceTimersByTime(600)
    await flushPromises()

    const newSrc = wrapper.find('img[alt="Poster preview"]').attributes('src')!
    expect(newSrc).toContain('ratings_limit=5')
    expect(newSrc).not.toBe(initialSrc)
  })

  it('debounces rapid changes — only last value takes effect', async () => {
    const wrapper = mountForm()
    const limitInput = wrapper.find('input[type="number"]')

    // Rapid changes
    await limitInput.setValue(1)
    vi.advanceTimersByTime(200)
    await limitInput.setValue(4)
    vi.advanceTimersByTime(200)
    await limitInput.setValue(7)

    // Advance past final debounce
    vi.advanceTimersByTime(600)
    await flushPromises()

    const src = wrapper.find('img[alt="Poster preview"]').attributes('src')!
    expect(src).toContain('ratings_limit=7')
  })

  it('shows loading state while preview loads', () => {
    const wrapper = mountForm()
    // previewLoading starts true on mount (updatePreview is called)
    const spinner = wrapper.find('.animate-spin')
    expect(spinner.exists()).toBe(true)

    // Image should be hidden while loading (v-show)
    const img = wrapper.find('img[alt="Poster preview"]')
    expect(img.isVisible()).toBe(false)
  })

  it('hides loading spinner and shows image after load', async () => {
    const wrapper = mountForm()
    const img = wrapper.find('img[alt="Poster preview"]')

    // Simulate the image load event
    await img.trigger('load')
    await flushPromises()

    expect(wrapper.find('.animate-spin').exists()).toBe(false)
    expect(img.isVisible()).toBe(true)
  })

  it('shows error message when preview fails to load', async () => {
    const wrapper = mountForm()
    const img = wrapper.find('img[alt="Poster preview"]')

    // Simulate image error
    await img.trigger('error')
    await flushPromises()

    expect(wrapper.text()).toContain('Failed to load preview')
  })

  it('preview URL uses BASE_URL from api module', () => {
    const wrapper = mountForm()
    const src = wrapper.find('img[alt="Poster preview"]').attributes('src')!
    expect(src.startsWith('http://test-api/api/preview/poster')).toBe(true)
  })
})
