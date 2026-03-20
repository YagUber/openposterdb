export const FREE_API_KEY = 't0-free-rpdb'

export const LANGUAGES = [
  { code: 'ar', name: 'Arabic' },
  { code: 'bg', name: 'Bulgarian' },
  { code: 'ca', name: 'Catalan' },
  { code: 'zh', name: 'Chinese' },
  { code: 'hr', name: 'Croatian' },
  { code: 'cs', name: 'Czech' },
  { code: 'da', name: 'Danish' },
  { code: 'nl', name: 'Dutch' },
  { code: 'en', name: 'English' },
  { code: 'fi', name: 'Finnish' },
  { code: 'fr', name: 'French' },
  { code: 'de', name: 'German' },
  { code: 'el', name: 'Greek' },
  { code: 'he', name: 'Hebrew' },
  { code: 'hi', name: 'Hindi' },
  { code: 'hu', name: 'Hungarian' },
  { code: 'id', name: 'Indonesian' },
  { code: 'it', name: 'Italian' },
  { code: 'ja', name: 'Japanese' },
  { code: 'ko', name: 'Korean' },
  { code: 'ms', name: 'Malay' },
  { code: 'no', name: 'Norwegian' },
  { code: 'fa', name: 'Persian' },
  { code: 'pl', name: 'Polish' },
  { code: 'pt', name: 'Portuguese' },
  { code: 'ro', name: 'Romanian' },
  { code: 'ru', name: 'Russian' },
  { code: 'sr', name: 'Serbian' },
  { code: 'sk', name: 'Slovak' },
  { code: 'sl', name: 'Slovenian' },
  { code: 'es', name: 'Spanish' },
  { code: 'sv', name: 'Swedish' },
  { code: 'th', name: 'Thai' },
  { code: 'tr', name: 'Turkish' },
  { code: 'uk', name: 'Ukrainian' },
  { code: 'vi', name: 'Vietnamese' },
] as const

export const ALL_RATING_SOURCES = [
  { key: 'imdb', label: 'IMDb', color: '#b4910f' },
  { key: 'tmdb', label: 'TMDB', color: '#019b58' },
  { key: 'rt', label: 'Rotten Tomatoes (Critics)', color: '#b92308' },
  { key: 'rta', label: 'Rotten Tomatoes (Audience)', color: '#b92308' },
  { key: 'mc', label: 'Metacritic', color: '#4b9626' },
  { key: 'trakt', label: 'Trakt', color: '#af0f2d' },
  { key: 'lb', label: 'Letterboxd', color: '#009b58' },
  { key: 'mal', label: 'MyAnimeList', color: '#223c78' },
] as const

export const DEFAULT_RATINGS_ORDER = 'mal,imdb,lb,rt,mc,rta,tmdb,trakt'

export function parseRatingsOrder(order: string): string[] {
  const keys = order ? order.split(',').map(k => k.trim()).filter(Boolean) : []
  const allKeys = ALL_RATING_SOURCES.map(s => s.key)
  for (const k of allKeys) {
    if (!keys.includes(k)) keys.push(k)
  }
  return keys
}
