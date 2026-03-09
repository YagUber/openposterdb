<script setup lang="ts">
import { ref, computed, onUnmounted } from 'vue'
import { useRoute, useRouter } from 'vue-router'
import { useQuery } from '@tanstack/vue-query'
import { adminApi } from '@/lib/api'
import { Eye } from 'lucide-vue-next'
import RefreshButton from '@/components/RefreshButton.vue'
import { Button } from '@/components/ui/button'
import { Skeleton } from '@/components/ui/skeleton'
import {
  Dialog,
  DialogContent,
  DialogHeader,
  DialogTitle,
} from '@/components/ui/dialog'
import {
  Table,
  TableBody,
  TableCell,
  TableHead,
  TableHeader,
  TableRow,
} from '@/components/ui/table'

interface PosterMeta {
  cache_key: string
  release_date: string | null
  created_at: number
  updated_at: number
}

interface PostersResponse {
  items: PosterMeta[]
  total: number
  page: number
  page_size: number
}

const route = useRoute()
const router = useRouter()

const page = computed({
  get: () => {
    const p = Number(route.query.page)
    return p > 0 ? p : 1
  },
  set: (v: number) => {
    router.replace({ query: { ...route.query, page: v === 1 ? undefined : String(v) } })
  },
})
const pageSize = 50

const { data, isPending, isFetching, refetch } = useQuery<PostersResponse>({
  queryKey: computed(() => ['admin', 'posters', page.value]),
  queryFn: async () => {
    const res = await adminApi.getPosters(page.value, pageSize)
    if (!res.ok) throw new Error('Failed to fetch posters')
    return res.json()
  },
})

const previewOpen = ref(false)
const previewKey = ref('')
const previewUrl = ref<string | null>(null)
const previewLoading = ref(false)

async function openPreview(cacheKey: string) {
  previewKey.value = cacheKey
  previewOpen.value = true
  previewLoading.value = true
  previewUrl.value = null

  try {
    const res = await adminApi.getPosterImage(cacheKey)
    if (res.ok) {
      const blob = await res.blob()
      previewUrl.value = URL.createObjectURL(blob)
    }
  } finally {
    previewLoading.value = false
  }
}

function closePreview() {
  previewOpen.value = false
  if (previewUrl.value) {
    URL.revokeObjectURL(previewUrl.value)
    previewUrl.value = null
  }
}

onUnmounted(() => {
  if (previewUrl.value) URL.revokeObjectURL(previewUrl.value)
})

function parseKey(cacheKey: string) {
  const idx = cacheKey.indexOf('/')
  if (idx === -1) return { idType: cacheKey, idValue: '' }
  return { idType: cacheKey.slice(0, idx), idValue: cacheKey.slice(idx + 1) }
}

function formatDate(epoch: number) {
  return new Date(epoch * 1000).toLocaleDateString()
}

function relativeTime(epoch: number) {
  const diff = Date.now() / 1000 - epoch
  if (diff < 60) return 'just now'
  if (diff < 3600) return `${Math.floor(diff / 60)}m ago`
  if (diff < 86400) return `${Math.floor(diff / 3600)}h ago`
  return `${Math.floor(diff / 86400)}d ago`
}

const totalPages = computed(() => data.value ? Math.ceil(data.value.total / data.value.page_size) : 0)

function prevPage() {
  if (page.value > 1) page.value--
}

function nextPage() {
  if (page.value < totalPages.value) page.value++
}
</script>

<template>
  <div class="space-y-4">
    <div class="flex justify-end">
      <RefreshButton :fetching="isFetching" @refresh="refetch()" />
    </div>
    <div v-if="isPending" class="space-y-3">
      <Skeleton v-for="i in 5" :key="i" class="h-10 w-full" />
    </div>
    <template v-else-if="data">
      <Table>
        <TableHeader>
          <TableRow>
            <TableHead class="w-10"></TableHead>
            <TableHead>ID Type</TableHead>
            <TableHead>ID Value</TableHead>
            <TableHead>Release Date</TableHead>
            <TableHead>Last Updated</TableHead>
            <TableHead>Created</TableHead>
          </TableRow>
        </TableHeader>
        <TableBody>
          <TableRow v-if="data.items.length === 0">
            <TableCell colspan="6" class="text-center text-muted-foreground">No posters cached yet.</TableCell>
          </TableRow>
          <TableRow v-for="item in data.items" :key="item.cache_key" class="cursor-pointer" @click="openPreview(item.cache_key)">
            <TableCell>
              <Eye class="size-4 text-muted-foreground" />
            </TableCell>
            <TableCell class="font-mono text-xs">{{ parseKey(item.cache_key).idType }}</TableCell>
            <TableCell class="font-mono text-xs">{{ parseKey(item.cache_key).idValue }}</TableCell>
            <TableCell>{{ item.release_date || '—' }}</TableCell>
            <TableCell>{{ relativeTime(item.updated_at) }}</TableCell>
            <TableCell>{{ formatDate(item.created_at) }}</TableCell>
          </TableRow>
        </TableBody>
      </Table>
      <div class="flex items-center justify-between">
        <p class="text-sm text-muted-foreground">
          {{ data.total }} poster{{ data.total === 1 ? '' : 's' }} total
        </p>
        <div class="flex items-center gap-2">
          <Button variant="outline" size="sm" :disabled="page <= 1" @click="prevPage">Previous</Button>
          <span class="text-sm">Page {{ page }} of {{ totalPages }}</span>
          <Button variant="outline" size="sm" :disabled="page >= totalPages" @click="nextPage">Next</Button>
        </div>
      </div>
    </template>

    <Dialog :open="previewOpen" @update:open="(v: boolean) => { if (!v) closePreview() }">
      <DialogContent class="max-w-md">
        <DialogHeader>
          <DialogTitle class="font-mono text-sm">{{ previewKey }}</DialogTitle>
        </DialogHeader>
        <div class="flex items-center justify-center min-h-[200px]">
          <Skeleton v-if="previewLoading" class="h-[400px] w-[270px] rounded-md" />
          <img
            v-else-if="previewUrl"
            :src="previewUrl"
            :alt="previewKey"
            class="max-h-[70vh] rounded-md object-contain"
          />
          <p v-else class="text-sm text-muted-foreground">Failed to load poster</p>
        </div>
      </DialogContent>
    </Dialog>
  </div>
</template>
