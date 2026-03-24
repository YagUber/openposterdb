import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest'
import { mount, flushPromises, VueWrapper } from '@vue/test-utils'
import { createPinia, setActivePinia } from 'pinia'
import FreeApiKeyCard from '@/components/FreeApiKeyCard.vue'
import { useAuthStore } from '@/stores/auth'

vi.mock('@/stores/auth', async (importOriginal) => {
  const actual = await importOriginal<typeof import('@/stores/auth')>()
  return actual
})

const SelectStub = {
  name: 'Select',
  template: '<div data-stub="select"><slot /></div>',
  props: ['modelValue'],
  emits: ['update:modelValue'],
}

function mountCard(freeApiKeyEnabled = true) {
  const pinia = createPinia()
  setActivePinia(pinia)
  const auth = useAuthStore()
  auth.freeApiKeyEnabled = freeApiKeyEnabled

  return mount(FreeApiKeyCard, {
    global: {
      plugins: [pinia],
      stubs: {
        Select: SelectStub,
        SelectTrigger: { template: '<span :id="id"><slot /></span>', props: ['id', 'ariaLabel', 'class'] },
        SelectValue: { template: '<span>{{ placeholder }}</span>', props: ['placeholder'] },
        SelectContent: { template: '<span><slot /></span>' },
        SelectItem: { template: '<span><slot /></span>', props: ['value'] },
        Collapsible: { template: '<div><slot /></div>', props: ['open'] },
        CollapsibleTrigger: { template: '<div><slot /></div>', props: ['asChild'] },
        CollapsibleContent: { template: '<div><slot /></div>' },
        Input: {
          name: 'Input',
          template: '<input :value="modelValue" :placeholder="placeholder" @input="$emit(\'update:modelValue\', $event.target.value)" />',
          props: ['modelValue', 'type', 'placeholder', 'required', 'id'],
        },
        Button: {
          template: '<button :disabled="disabled" :type="type" @click="$emit(\'click\')"><slot /></button>',
          props: ['disabled', 'variant', 'size', 'type'],
        },
        ChevronRight: { template: '<svg />' },
        Loader2: { template: '<svg />' },
      },
    },
  })
}

/**
 * Set a Select's value by finding its SelectTrigger via id,
 * then emitting on the parent Select component.
 */
async function setSelectById(wrapper: VueWrapper, triggerId: string, value: string) {
  const trigger = wrapper.find(`#${triggerId}`)
  if (!trigger.exists()) {
    throw new Error(`SelectTrigger with id="${triggerId}" not found`)
  }
  // Walk up to find the parent Select stub component
  const selectComponents = wrapper.findAllComponents(SelectStub)
  const parent = selectComponents.find(s => s.find(`#${triggerId}`).exists())
  if (!parent) {
    throw new Error(`Parent Select for trigger id="${triggerId}" not found`)
  }
  parent.vm.$emit('update:modelValue', value)
  await flushPromises()
}

/** Get the curl example code element (the last code element). */
function findCurlCode(wrapper: VueWrapper) {
  const codes = wrapper.findAll('code')
  return codes[codes.length - 1]
}

describe('FreeApiKeyCard', () => {
  beforeEach(() => {
    vi.clearAllMocks()
  })

  afterEach(() => {
    vi.restoreAllMocks()
  })

  it('renders nothing when freeApiKeyEnabled is false', () => {
    const wrapper = mountCard(false)
    expect(wrapper.text()).toBe('')
  })

  it('renders card with FREE_API_KEY code when enabled', () => {
    const wrapper = mountCard(true)
    expect(wrapper.text()).toContain('t0-free-rpdb')
    expect(wrapper.text()).toContain('Free API Key Available')
  })

  it('curlExample uses .jpg for poster and .png for logo', async () => {
    const wrapper = mountCard(true)
    expect(findCurlCode(wrapper).text()).toContain('.jpg')

    await setSelectById(wrapper, 'free-image-type', 'logo')
    expect(findCurlCode(wrapper).text()).toContain('.png')
  })

  it('sizeOptions excludes small for poster, includes small for backdrop', async () => {
    const wrapper = mountCard(true)
    // Default is poster — set "small" then switch imageType to trigger the watch
    await setSelectById(wrapper, 'free-image-size', 'small')
    // Switch imageType away to trigger the reset (small invalid for logo)
    await setSelectById(wrapper, 'free-image-type', 'logo')
    expect(findCurlCode(wrapper).text()).not.toContain('imageSize=small')

    // Switch to backdrop — "small" should be valid and persist
    await setSelectById(wrapper, 'free-image-type', 'backdrop')
    await setSelectById(wrapper, 'free-image-size', 'small')
    expect(findCurlCode(wrapper).text()).toContain('imageSize=small')
  })

  it('resets imageSize to default when switching imageType invalidates current size', async () => {
    const wrapper = mountCard(true)

    // Switch to backdrop
    await setSelectById(wrapper, 'free-image-type', 'backdrop')
    // Set size to small (valid for backdrop)
    await setSelectById(wrapper, 'free-image-size', 'small')
    expect(findCurlCode(wrapper).text()).toContain('imageSize=small')

    // Switch back to poster — "small" is invalid, should reset to default
    await setSelectById(wrapper, 'free-image-type', 'poster')
    expect(findCurlCode(wrapper).text()).not.toContain('imageSize=small')
  })

  it('handleFetch creates blob URL on success', async () => {
    const blobUrl = 'blob:http://localhost/fake'
    vi.spyOn(URL, 'createObjectURL').mockReturnValue(blobUrl)
    vi.spyOn(URL, 'revokeObjectURL').mockImplementation(() => {})
    vi.stubGlobal(
      'fetch',
      vi.fn().mockResolvedValue({
        ok: true,
        blob: () => Promise.resolve(new Blob(['img'], { type: 'image/jpeg' })),
      }),
    )

    const wrapper = mountCard(true)
    const form = wrapper.find('form')
    await form.trigger('submit')
    await flushPromises()

    expect(URL.createObjectURL).toHaveBeenCalled()
    const img = wrapper.find('img[alt="Fetched result"]')
    expect(img.exists()).toBe(true)
    expect(img.attributes('src')).toBe(blobUrl)
  })

  it('handleFetch shows "Not found" error on 404', async () => {
    vi.stubGlobal(
      'fetch',
      vi.fn().mockResolvedValue({ ok: false, status: 404 }),
    )

    const wrapper = mountCard(true)
    await wrapper.find('form').trigger('submit')
    await flushPromises()

    expect(wrapper.text()).toContain('Not found')
  })

  it('handleFetch shows generic error on network failure', async () => {
    vi.stubGlobal(
      'fetch',
      vi.fn().mockRejectedValue(new TypeError('Failed to fetch')),
    )

    const wrapper = mountCard(true)
    await wrapper.find('form').trigger('submit')
    await flushPromises()

    expect(wrapper.text()).toContain('Failed to fetch')
  })

  it('queryString includes lang, imageSize params when set', async () => {
    const wrapper = mountCard(true)

    await setSelectById(wrapper, 'free-lang', 'en')
    await setSelectById(wrapper, 'free-image-size', 'large')

    const curlText = findCurlCode(wrapper).text()
    expect(curlText).toContain('lang=en')
    expect(curlText).toContain('imageSize=large')
  })

  it('idPlaceholder changes per idType', async () => {
    const wrapper = mountCard(true)

    const getPlaceholder = () => wrapper.find('input:not([type="checkbox"])').attributes('placeholder')
    expect(getPlaceholder()).toBe('tt0013442')

    await setSelectById(wrapper, 'free-id-type', 'tmdb')
    expect(getPlaceholder()).toBe('movie-872585 or episode-1396-S1E1')

    await setSelectById(wrapper, 'free-id-type', 'tvdb')
    expect(getPlaceholder()).toBe('253573')
  })

  it('resets poster-only controls when switching away from poster', async () => {
    const wrapper = mountCard(true)

    // Set poster-only controls and general image controls
    await setSelectById(wrapper, 'free-poster-position', 'tl')
    await setSelectById(wrapper, 'free-badge-direction', 'v')
    await setSelectById(wrapper, 'free-image-source', 'f')
    await setSelectById(wrapper, 'free-textless', 'true')
    expect(findCurlCode(wrapper).text()).toContain('position=tl')
    expect(findCurlCode(wrapper).text()).toContain('badge_direction=v')
    expect(findCurlCode(wrapper).text()).toContain('image_source=f')
    expect(findCurlCode(wrapper).text()).toContain('textless=true')

    // Switch to logo — poster-only controls should reset, source persists
    await setSelectById(wrapper, 'free-image-type', 'logo')
    expect(findCurlCode(wrapper).text()).not.toContain('position=')
    expect(findCurlCode(wrapper).text()).not.toContain('badge_direction=')
    expect(findCurlCode(wrapper).text()).not.toContain('textless=')
    expect(findCurlCode(wrapper).text()).toContain('image_source=f')

    // Switch back to poster — poster-only controls at defaults, source still set
    await setSelectById(wrapper, 'free-image-type', 'poster')
    expect(findCurlCode(wrapper).text()).not.toContain('position=')
    expect(findCurlCode(wrapper).text()).not.toContain('badge_direction=')
    expect(findCurlCode(wrapper).text()).not.toContain('textless=')
    expect(findCurlCode(wrapper).text()).toContain('image_source=f')
  })

  // --- Episode support ---

  it('episode queryString includes badge_direction, position, and blur', async () => {
    const wrapper = mountCard(true)
    await setSelectById(wrapper, 'free-image-type', 'episode')
    await setSelectById(wrapper, 'free-badge-direction', 'v')
    await setSelectById(wrapper, 'free-poster-position', 'tr')
    await setSelectById(wrapper, 'free-blur', 'true')

    const curlText = findCurlCode(wrapper).text()
    expect(curlText).toContain('badge_direction=v')
    expect(curlText).toContain('position=tr')
    expect(curlText).toContain('blur=true')
  })

  it('blur selector only appears for episode image type', async () => {
    const wrapper = mountCard(true)

    // Default is poster — no blur select
    expect(wrapper.find('#free-blur').exists()).toBe(false)

    // Switch to episode — blur should appear
    await setSelectById(wrapper, 'free-image-type', 'episode')
    expect(wrapper.find('#free-blur').exists()).toBe(true)

    // Switch to backdrop — blur should disappear
    await setSelectById(wrapper, 'free-image-type', 'backdrop')
    expect(wrapper.find('#free-blur').exists()).toBe(false)
  })

  it('switching from episode to logo resets blur', async () => {
    const wrapper = mountCard(true)

    await setSelectById(wrapper, 'free-image-type', 'episode')
    await setSelectById(wrapper, 'free-blur', 'true')
    expect(findCurlCode(wrapper).text()).toContain('blur=true')

    // Switch to logo — blur should reset
    await setSelectById(wrapper, 'free-image-type', 'logo')
    expect(findCurlCode(wrapper).text()).not.toContain('blur=')
  })

  it('episode uses .jpg extension in curl example', async () => {
    const wrapper = mountCard(true)
    await setSelectById(wrapper, 'free-image-type', 'episode')

    const curlText = findCurlCode(wrapper).text()
    expect(curlText).toContain('episode-default/')
    expect(curlText).toContain('.jpg')
  })

  it('sizeOptions includes small for episode', async () => {
    const wrapper = mountCard(true)
    await setSelectById(wrapper, 'free-image-type', 'episode')
    await setSelectById(wrapper, 'free-image-size', 'small')
    expect(findCurlCode(wrapper).text()).toContain('imageSize=small')
  })

  it('episode keeps position and badge_direction controls', async () => {
    const wrapper = mountCard(true)
    await setSelectById(wrapper, 'free-image-type', 'episode')

    expect(wrapper.find('#free-poster-position').exists()).toBe(true)
    expect(wrapper.find('#free-badge-direction').exists()).toBe(true)
  })

  it('episode does not show textless control', async () => {
    const wrapper = mountCard(true)
    await setSelectById(wrapper, 'free-image-type', 'episode')

    expect(wrapper.find('#free-textless').exists()).toBe(false)
  })
})
