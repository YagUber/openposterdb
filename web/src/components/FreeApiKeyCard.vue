<script setup lang="ts">
import { ref, computed, watch, onUnmounted } from 'vue'
import { useAuthStore } from '@/stores/auth'
import { FREE_API_KEY, LANGUAGES } from '@/lib/constants'
import { Button } from '@/components/ui/button'
import { Input } from '@/components/ui/input'
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from '@/components/ui/select'
import { Collapsible, CollapsibleContent, CollapsibleTrigger } from '@/components/ui/collapsible'
import { ChevronRight, Loader2 } from 'lucide-vue-next'

const isOpen = ref(false)

const auth = useAuthStore()

const idType = ref<'imdb' | 'tmdb' | 'tvdb'>('imdb')
const imageType = ref<'poster' | 'logo' | 'backdrop'>('poster')
const idValue = ref('tt0013442')
const lang = ref('any')
const imageSize = ref<'default' | 'small' | 'medium' | 'large' | 'verylarge'>('default')
const fetchError = ref('')
const fetchLoading = ref(false)
const resultUrl = ref('')
const resultImageType = ref<'poster' | 'logo' | 'backdrop'>('poster')

const sizeOptions = computed(() => {
  if (imageType.value === 'backdrop') {
    return [
      { value: 'default', label: 'Size: default' },
      { value: 'small', label: 'Small' },
      { value: 'medium', label: 'Medium' },
      { value: 'large', label: 'Large' },
    ]
  }
  return [
    { value: 'default', label: 'Size: default' },
    { value: 'medium', label: 'Medium' },
    { value: 'large', label: 'Large' },
    { value: 'verylarge', label: 'Very Large' },
  ]
})

// Reset size when switching image type if the current size is invalid
watch(imageType, () => {
  const validValues = sizeOptions.value.map(o => o.value)
  if (!validValues.includes(imageSize.value)) {
    imageSize.value = 'default'
  }
})

const apiBase = import.meta.env.VITE_API_URL || ''

onUnmounted(() => {
  if (resultUrl.value) URL.revokeObjectURL(resultUrl.value)
})

const idPlaceholder = computed(() => {
  if (idType.value === 'imdb') return 'tt0013442'
  if (idType.value === 'tmdb') return 'movie-872585'
  return '253573'
})

const queryString = computed(() => {
  const params = new URLSearchParams()
  const langVal = lang.value === 'any' ? '' : lang.value
  if (langVal.trim()) params.set('lang', langVal.trim())
  const sizeVal = imageSize.value === 'default' ? '' : imageSize.value
  if (sizeVal) params.set('imageSize', sizeVal)
  const qs = params.toString()
  return qs ? `?${qs}` : ''
})

const curlExample = computed(() => {
  const id = idValue.value.trim() || idPlaceholder.value
  const ext = imageType.value === 'logo' ? 'png' : 'jpg'
  return `curl -o ${imageType.value}.${ext} "${window.location.origin}/${FREE_API_KEY}/${idType.value}/${imageType.value}-default/${id}.${ext}${queryString.value}"`
})

const resultClass = computed(() => {
  if (resultImageType.value === 'logo') return 'max-w-[400px]'
  if (resultImageType.value === 'backdrop') return 'max-w-[500px] rounded-lg shadow-lg'
  return 'max-w-[200px] rounded-lg shadow-lg'
})

async function handleFetch() {
  const id = idValue.value.trim()
  if (!id) return

  fetchError.value = ''
  fetchLoading.value = true

  const prevUrl = resultUrl.value

  const ext = imageType.value === 'logo' ? 'png' : 'jpg'
  const url = `${apiBase}/${FREE_API_KEY}/${idType.value}/${imageType.value}-default/${id}.${ext}${queryString.value}`

  try {
    const res = await fetch(url)
    if (!res.ok) throw new Error(res.status === 404 ? 'Not found — check the ID and try again' : `Server error (${res.status})`)
    const blob = await res.blob()
    resultImageType.value = imageType.value
    resultUrl.value = URL.createObjectURL(blob)
    if (prevUrl) URL.revokeObjectURL(prevUrl)
  } catch (e) {
    fetchError.value = e instanceof Error && e.message ? e.message : 'Failed to fetch — check the ID and try again'
    if (prevUrl) URL.revokeObjectURL(prevUrl)
    resultUrl.value = ''
  } finally {
    fetchLoading.value = false
  }
}
</script>

<template>
  <div v-if="auth.freeApiKeyEnabled" class="rounded-lg border border-blue-500/30 bg-blue-500/5 p-4 space-y-3">
    <p class="text-sm font-medium">Free API Key Available</p>
    <p class="text-sm text-muted-foreground">
      Use the following key for poster serving (read-only, global defaults):
    </p>
    <code class="block text-sm font-mono bg-muted px-3 py-2 rounded select-all">{{ FREE_API_KEY }}</code>
    <Collapsible v-model:open="isOpen">
      <CollapsibleTrigger as-child>
        <button class="flex w-full items-center gap-2 text-sm font-medium text-muted-foreground hover:text-foreground transition-colors">
          <ChevronRight class="h-4 w-4 shrink-0 transition-transform duration-200" :class="{ 'rotate-90': isOpen }" />
          Try it out
        </button>
      </CollapsibleTrigger>
      <CollapsibleContent class="pt-3 space-y-3">
      <form class="flex flex-col gap-2" @submit.prevent="handleFetch">
        <div class="flex flex-col sm:flex-row gap-2">
          <Select v-model="idType">
            <SelectTrigger id="free-id-type" aria-label="ID type" class="bg-background">
              <SelectValue placeholder="ID type" />
            </SelectTrigger>
            <SelectContent>
              <SelectItem value="imdb">IMDb</SelectItem>
              <SelectItem value="tmdb">TMDb</SelectItem>
              <SelectItem value="tvdb">TVDB</SelectItem>
            </SelectContent>
          </Select>
          <Select v-model="imageType">
            <SelectTrigger id="free-image-type" aria-label="Image type" class="bg-background">
              <SelectValue placeholder="Image type" />
            </SelectTrigger>
            <SelectContent>
              <SelectItem value="poster">Poster</SelectItem>
              <SelectItem value="logo">Logo</SelectItem>
              <SelectItem value="backdrop">Backdrop</SelectItem>
            </SelectContent>
          </Select>
          <Input
            id="free-id-value"
            v-model="idValue"
            type="text"
            :placeholder="idPlaceholder"
            required
            class="flex-1 min-w-0 font-mono bg-background"
          />
        </div>
        <div class="flex flex-col sm:flex-row gap-2 items-start sm:items-center">
          <Select v-model="imageSize">
            <SelectTrigger id="free-image-size" aria-label="Image size" class="bg-background">
              <SelectValue placeholder="Size: default" />
            </SelectTrigger>
            <SelectContent>
              <SelectItem v-for="opt in sizeOptions" :key="opt.value" :value="opt.value">
                {{ opt.label }}
              </SelectItem>
            </SelectContent>
          </Select>
          <Select v-model="lang">
            <SelectTrigger id="free-lang" aria-label="Language" class="bg-background">
              <SelectValue placeholder="Language: any" />
            </SelectTrigger>
            <SelectContent>
              <SelectItem value="any">Language: any</SelectItem>
              <SelectItem v-for="l in LANGUAGES" :key="l.code" :value="l.code">
                {{ l.code }} - {{ l.name }}
              </SelectItem>
            </SelectContent>
          </Select>
        </div>
        <code class="block text-xs font-mono bg-muted px-3 py-2 rounded text-muted-foreground break-all select-all">{{ curlExample }}</code>
        <div class="flex justify-center">
          <Button type="submit" size="lg" :disabled="fetchLoading">
            <Loader2 v-if="fetchLoading" class="h-4 w-4 animate-spin" />
            <span v-else>Fetch</span>
          </Button>
        </div>
      </form>
      <p v-if="fetchError" class="text-sm text-destructive">{{ fetchError }}</p>
      <div v-if="resultUrl" class="flex justify-center pt-2 overflow-hidden">
        <img
          :src="resultUrl"
          alt="Fetched result"
          :class="['w-full', resultClass]"
        />
      </div>
      </CollapsibleContent>
    </Collapsible>
  </div>
</template>
