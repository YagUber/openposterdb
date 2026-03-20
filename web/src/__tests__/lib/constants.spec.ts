import { describe, it, expect } from 'vitest'
import { ALL_RATING_SOURCES, DEFAULT_RATINGS_ORDER, parseRatingsOrder } from '@/lib/constants'

describe('ALL_RATING_SOURCES', () => {
  it('has 8 rating sources', () => {
    expect(ALL_RATING_SOURCES).toHaveLength(8)
  })

  it('each source has key, label, and color', () => {
    for (const source of ALL_RATING_SOURCES) {
      expect(source.key).toBeTruthy()
      expect(source.label).toBeTruthy()
      expect(source.color).toMatch(/^#[0-9a-f]{6}$/i)
    }
  })

  it('keys are unique', () => {
    const keys = ALL_RATING_SOURCES.map(s => s.key)
    expect(new Set(keys).size).toBe(keys.length)
  })
})

describe('parseRatingsOrder', () => {
  it('returns keys in given order followed by missing keys', () => {
    const result = parseRatingsOrder('imdb,tmdb')
    expect(result[0]).toBe('imdb')
    expect(result[1]).toBe('tmdb')
    // All 8 keys should be present
    expect(result).toHaveLength(8)
  })

  it('appends missing keys for partial input', () => {
    const result = parseRatingsOrder('rt')
    expect(result[0]).toBe('rt')
    // All other keys should follow
    const allKeys = ALL_RATING_SOURCES.map(s => s.key)
    for (const key of allKeys) {
      expect(result).toContain(key)
    }
  })

  it('handles empty string by returning all keys', () => {
    const result = parseRatingsOrder('')
    const allKeys = ALL_RATING_SOURCES.map(s => s.key)
    expect(result).toHaveLength(allKeys.length)
    for (const key of allKeys) {
      expect(result).toContain(key)
    }
  })

  it('preserves the full default order', () => {
    const result = parseRatingsOrder(DEFAULT_RATINGS_ORDER)
    expect(result).toEqual(DEFAULT_RATINGS_ORDER.split(','))
  })

  it('trims whitespace around keys', () => {
    const result = parseRatingsOrder('imdb , tmdb')
    expect(result[0]).toBe('imdb')
    expect(result[1]).toBe('tmdb')
  })

  it('does not include a key twice when already present in input', () => {
    // parseRatingsOrder appends missing keys — keys already in the input should not be appended again
    const result = parseRatingsOrder('imdb,tmdb')
    const imdbCount = result.filter((k: string) => k === 'imdb').length
    expect(imdbCount).toBe(1)
  })
})
