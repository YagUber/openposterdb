<script setup lang="ts">
import { Button } from "@/components/ui/button";
import { BookOpen, Github } from "lucide-vue-next";

withDefaults(defineProps<{ primaryLabel?: string; primaryTo?: string }>(), {
  primaryLabel: "Sign in",
  primaryTo: "/login",
});
</script>

<template>
  <div class="flex flex-wrap items-center justify-center gap-3">
    <Button as-child size="lg">
      <router-link :to="primaryTo">{{ primaryLabel }}</router-link>
    </Button>
    <Button as-child variant="outline" size="lg">
      <router-link to="/docs">
        <BookOpen class="h-5 w-5" />
        API Docs
      </router-link>
    </Button>
    <Button as-child variant="outline" size="lg">
      <a
        href="https://github.com/PNRxA/openposterdb"
        target="_blank"
        rel="noopener noreferrer"
      >
        <Github class="h-5 w-5" />
        GitHub
      </a>
    </Button>
  </div>
</template>
