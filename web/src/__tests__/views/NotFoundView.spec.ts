import { describe, it, expect } from 'vitest'
import { shallowMount } from '@vue/test-utils'
import NotFoundView from '@/views/NotFoundView.vue'
import NavButtons from '@/components/NavButtons.vue'

describe('NotFoundView', () => {
  function mountView() {
    return shallowMount(NotFoundView, {
      global: {
        stubs: {
          'router-link': {
            template: '<a :href="to"><slot /></a>',
            props: ['to'],
          },
        },
      },
    })
  }

  it('renders 404 heading', () => {
    const wrapper = mountView()
    expect(wrapper.find('h1').text()).toBe('404')
  })

  it('renders descriptive message', () => {
    const wrapper = mountView()
    expect(wrapper.text()).toContain("didn't make the final cut")
  })

  it('renders NavButtons with Go home link', () => {
    const wrapper = mountView()
    const navButtons = wrapper.findComponent(NavButtons)
    expect(navButtons.exists()).toBe(true)
    expect(navButtons.props('primaryLabel')).toBe('Go home')
    expect(navButtons.props('primaryTo')).toBe('/')
  })
})
