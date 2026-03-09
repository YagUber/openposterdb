<script setup lang="ts">
import { ref, watch } from 'vue'
import { RefreshCw, Check } from 'lucide-vue-next'
import { Button } from '@/components/ui/button'

const props = defineProps<{
  fetching: boolean
}>()

const emit = defineEmits<{
  refresh: []
}>()

const showCheck = ref(false)
let timeout: ReturnType<typeof setTimeout> | null = null

watch(() => props.fetching, (now, was) => {
  if (was && !now) {
    showCheck.value = true
    if (timeout) clearTimeout(timeout)
    timeout = setTimeout(() => {
      showCheck.value = false
    }, 1500)
  }
})
</script>

<template>
  <Button variant="outline" size="sm" :disabled="fetching" @click="emit('refresh')">
    <span class="relative size-4">
      <Transition
        enter-active-class="transition duration-200 ease-out"
        enter-from-class="opacity-0 scale-50"
        enter-to-class="opacity-100 scale-100"
        leave-active-class="transition duration-150 ease-in"
        leave-from-class="opacity-100 scale-100"
        leave-to-class="opacity-0 scale-50"
      >
        <Check v-if="showCheck" class="absolute inset-0 size-4 text-green-500" />
        <RefreshCw v-else class="absolute inset-0 size-4" :class="{ 'animate-spin': fetching }" />
      </Transition>
    </span>
    Refresh
  </Button>
</template>
