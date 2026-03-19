<script setup lang="ts">
import NavButtons from "@/components/NavButtons.vue";
import BlurImage from "@/components/BlurImage.vue";

const posters = [
  { src: "/examples/nosferatu.webp", label: "Nosferatu (1922)", rotate: "-6deg", delay: "0s" },
  { src: "/examples/metropolis.webp", label: "Metropolis (1927)", rotate: "4deg", delay: "0.1s" },
  { src: "/examples/caligari.webp", label: "Dr. Caligari (1920)", rotate: "-3deg", delay: "0.2s" },
  { src: "/examples/phantom.webp", label: "Phantom of the Opera (1925)", rotate: "5deg", delay: "0.3s" },
  { src: "/examples/trip-to-moon.webp", label: "A Trip to the Moon (1902)", rotate: "-5deg", delay: "0.4s" },
];
</script>

<template>
  <main class="min-h-screen flex flex-col items-center justify-center px-4 py-16">
    <div class="max-w-2xl w-full text-center space-y-8">
      <!-- Scattered posters -->
      <div class="flex justify-center items-end gap-3 sm:gap-4">
        <div
          v-for="(p, i) in posters"
          :key="p.src"
          class="poster-card"
          :class="{ 'hidden sm:block': i === 0 || i === posters.length - 1 }"
          :style="{ '--rotate': p.rotate, '--delay': p.delay }"
        >
          <BlurImage
            :src="p.src"
            :alt="p.label"
            :width="96"
            :height="144"
            class="rounded-lg shadow-xl opacity-40"
          />
        </div>
      </div>

      <!-- Message -->
      <div class="space-y-2">
        <h1 class="text-7xl sm:text-8xl font-bold tracking-tight">404</h1>
        <p class="text-lg text-muted-foreground">
          This scene didn't make the final cut.
        </p>
      </div>

      <!-- Actions -->
      <NavButtons primary-label="Go home" primary-to="/" />
    </div>
  </main>
</template>

<style scoped>
.poster-card {
  transform: rotate(var(--rotate));
  animation: fade-up 0.5s ease-out both;
  animation-delay: var(--delay);
}

@keyframes fade-up {
  from {
    opacity: 0;
    transform: rotate(var(--rotate)) translateY(12px);
  }
  to {
    opacity: 1;
    transform: rotate(var(--rotate)) translateY(0);
  }
}
</style>
