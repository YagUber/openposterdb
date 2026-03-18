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
        SelectTrigger: { template: '<span><slot /></span>' },
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
 * Find all Select stub component wrappers.
 * Order in FreeApiKeyCard template: idType, imageType, imageSize, lang
 */
function findSelectComponents(wrapper: VueWrapper) {
  return wrapper.findAllComponents(SelectStub)
}

/** Set a Select stub's value by emitting update:modelValue directly. */
async function setSelect(wrapper: VueWrapper, index: number, value: string) {
  const selects = findSelectComponents(wrapper)
  selects[index].vm.$emit('update:modelValue', value)
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

    await setSelect(wrapper, 1, 'logo') // imageType
    expect(findCurlCode(wrapper).text()).toContain('.png')
  })

  it('sizeOptions excludes small for poster, includes small for backdrop', async () => {
    const wrapper = mountCard(true)
    // Default is poster — set "small" then switch imageType to trigger the watch
    await setSelect(wrapper, 2, 'small') // imageSize = small
    // Now switch imageType to poster again (same value) — watch doesn't fire
    // Instead, switch imageType away and back to trigger the reset
    await setSelect(wrapper, 1, 'logo') // triggers watch → small invalid for logo → reset
    expect(findCurlCode(wrapper).text()).not.toContain('imageSize=small')

    // Switch to backdrop — "small" should be valid and persist
    await setSelect(wrapper, 1, 'backdrop') // imageType = backdrop
    await setSelect(wrapper, 2, 'small') // imageSize = small
    expect(findCurlCode(wrapper).text()).toContain('imageSize=small')
  })

  it('resets imageSize to default when switching imageType invalidates current size', async () => {
    const wrapper = mountCard(true)

    // Switch to backdrop
    await setSelect(wrapper, 1, 'backdrop')
    // Set size to small (valid for backdrop)
    await setSelect(wrapper, 2, 'small')
    expect(findCurlCode(wrapper).text()).toContain('imageSize=small')

    // Switch back to poster — "small" is invalid, should reset to default
    await setSelect(wrapper, 1, 'poster')
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

    // Set lang
    await setSelect(wrapper, 3, 'en')
    // Set imageSize
    await setSelect(wrapper, 2, 'large')

    const curlText = findCurlCode(wrapper).text()
    expect(curlText).toContain('lang=en')
    expect(curlText).toContain('imageSize=large')
  })

  it('idPlaceholder changes per idType', async () => {
    const wrapper = mountCard(true)

    const getPlaceholder = () => wrapper.find('input:not([type="checkbox"])').attributes('placeholder')
    expect(getPlaceholder()).toBe('tt0013442')

    await setSelect(wrapper, 0, 'tmdb')
    expect(getPlaceholder()).toBe('movie-872585')

    await setSelect(wrapper, 0, 'tvdb')
    expect(getPlaceholder()).toBe('253573')
  })
})
