<script setup lang="ts">
import { ref, computed } from "vue";
import { placeholders } from "virtual:placeholders";

const props = withDefaults(defineProps<{
  src: string;
  alt: string;
  width: number;
  height: number;
  fit?: "cover" | "contain";
}>(), {
  fit: "cover",
});

const loaded = ref(false);
const placeholder = computed(() => placeholders[props.src]);
</script>

<template>
  <div class="relative overflow-hidden" :style="{ width: `${width}px`, maxWidth: '100%', aspectRatio: `${width}/${height}` }">
    <img
      v-if="placeholder"
      :src="placeholder"
      :alt="alt"
      :width="width"
      :height="height"
      class="absolute inset-0 w-full h-full blur-sm scale-105 transition-opacity duration-300"
      :class="[loaded ? 'opacity-0' : 'opacity-100', fit === 'contain' ? 'object-contain' : 'object-cover']"
      aria-hidden="true"
    />
    <img
      :src="src"
      :alt="alt"
      :width="width"
      :height="height"
      loading="lazy"
      class="absolute inset-0 w-full h-full transition-opacity duration-300"
      :class="[loaded ? 'opacity-100' : 'opacity-0', fit === 'contain' ? 'object-contain' : 'object-cover']"
      @load="loaded = true"
    />
  </div>
</template>
