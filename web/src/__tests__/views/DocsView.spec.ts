import { describe, it, expect, vi } from 'vitest'
import { shallowMount } from '@vue/test-utils'

vi.mock('@scalar/api-reference', () => ({
  ApiReference: {
    name: 'ApiReference',
    template: '<div class="api-reference" />',
    props: ['configuration'],
  },
}))

vi.mock('@scalar/api-reference/style.css', () => ({}))

import DocsView from '@/views/DocsView.vue'

describe('DocsView', () => {
  function mountView() {
    return shallowMount(DocsView, {
      global: {
        stubs: {
          'router-link': {
            template: '<a :href="to"><slot /></a>',
            props: ['to'],
          },
          ArrowLeft: { template: '<svg />' },
        },
      },
    })
  }

  it('renders topbar with OpenPosterDB text and API Reference subtitle', () => {
    const wrapper = mountView()
    expect(wrapper.text()).toContain('OpenPosterDB')
    expect(wrapper.text()).toContain('API Reference')
  })

  it('renders ApiReference component with correct config', () => {
    const wrapper = mountView()
    const apiRef = wrapper.findComponent({ name: 'ApiReference' })
    expect(apiRef.exists()).toBe(true)
    expect(apiRef.props('configuration')).toEqual(
      expect.objectContaining({
        url: '/api/openapi.json',
        hideClientButton: true,
      }),
    )
  })
})
