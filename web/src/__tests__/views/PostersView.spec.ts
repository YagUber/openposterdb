import { describe, it, expect, vi } from 'vitest'
import { shallowMount } from '@vue/test-utils'
import PostersView from '@/views/PostersView.vue'
import ImageListView from '@/components/ImageListView.vue'

vi.mock('@/lib/api', () => ({
  adminApi: {
    getPosters: vi.fn(),
    getPosterImage: vi.fn(),
    fetchPoster: vi.fn(),
  },
}))

describe('PostersView', () => {
  it('renders ImageListView with kind="poster"', () => {
    const wrapper = shallowMount(PostersView)

    const imageList = wrapper.findComponent(ImageListView)
    expect(imageList.exists()).toBe(true)
    expect(imageList.props('kind')).toBe('poster')
  })
})
