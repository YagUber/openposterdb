<script setup lang="ts">
import { version } from "../../package.json";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { Image, KeyRound, Zap, Shield, ExternalLink } from "lucide-vue-next";
import NavButtons from "@/components/NavButtons.vue";
import FreeApiKeyCard from "@/components/FreeApiKeyCard.vue";
import BlurImage from "@/components/BlurImage.vue";

const features = [
  {
    icon: Image,
    title: "Posters & Backdrops",
    desc: "Generate and serve posters, logos, and backdrops for movies, shows, episodes, and collections.",
  },
  {
    icon: KeyRound,
    title: "API Key Management",
    desc: "Create and manage API keys with per-key settings for different media servers.",
  },
  {
    icon: Zap,
    title: "Fast & Cached",
    desc: "In-memory caching and on-disk storage for instant poster delivery.",
  },
  {
    icon: Shield,
    title: "RPDB Compatible",
    desc: "Drop-in replacement for RPDB with full API compatibility.",
  },
];

const posters = [
  { src: "/examples/nosferatu.webp", label: "Nosferatu (1922)" },
  { src: "/examples/metropolis.webp", label: "Metropolis (1927)" },
  { src: "/examples/namakura-gatana.webp", alt: "Namakura Gatana (1917)", label: "Namakura Gatana (1917)" },
  {
    src: "/examples/caligari.webp",
    alt: "The Cabinet of Dr. Caligari (1920)",
    label: "Dr. Caligari (1920)",
  },
  {
    src: "/examples/phantom.webp",
    alt: "The Phantom of the Opera (1925)",
    label: "Phantom of the Opera (1925)",
  },
  {
    src: "/examples/trip-to-moon.webp",
    alt: "A Trip to the Moon (1902)",
    label: "A Trip to the Moon (1902)",
  },
  { src: "/examples/safety-last.webp", alt: "Safety Last! (1923)", label: "Safety Last! (1923)" },
  { src: "/examples/the-general.webp", label: "The General (1926)" },
];

const positions = [
  { src: "/examples/pos-tl.webp", label: "Top left" },
  { src: "/examples/pos-tc.webp", label: "Top center" },
  { src: "/examples/pos-tr.webp", label: "Top right" },
  { src: "/examples/pos-r.webp", label: "Right" },
  { src: "/examples/pos-bl.webp", label: "Bottom left" },
  { src: "/examples/pos-bc.webp", label: "Bottom center" },
  { src: "/examples/pos-br.webp", label: "Bottom right" },
  { src: "/examples/pos-l.webp", label: "Left" },
];

const styles = [
  { src: "/examples/style-h.webp", label: "Horizontal" },
  { src: "/examples/style-v.webp", label: "Vertical" },
];

const labels = [
  { src: "/examples/label-icon.webp", label: "Icons" },
  { src: "/examples/label-text.webp", label: "Text" },
  { src: "/examples/label-official.webp", label: "Official" },
];

const dataProviders = [
  { name: "TMDB", url: "https://www.themoviedb.org/", keyUrl: "https://www.themoviedb.org/settings/api", desc: "Movie & TV metadata and poster images" },
  { name: "MDBList", url: "https://mdblist.com/", keyUrl: "https://mdblist.com/preferences/", desc: "Aggregated ratings from multiple sources" },
  { name: "OMDb", url: "https://www.omdbapi.com/", keyUrl: "https://www.omdbapi.com/apikey.aspx", desc: "Alternative ratings source" },
  { name: "Fanart.tv", url: "https://fanart.tv/", keyUrl: "https://fanart.tv/get-an-api-key/", desc: "Fan art, logos, and backdrops" },
  { name: "RPDB", url: "https://ratingposterdb.com/", desc: "The original inspiration for this project" },
  { name: "Simple Icons", url: "https://simpleicons.org/", desc: "SVG icons for popular brands" },
];

const ratingSources = [
  { name: "IMDb", url: "https://www.imdb.com/" },
  { name: "TMDB", url: "https://www.themoviedb.org/" },
  { name: "Rotten Tomatoes", url: "https://www.rottentomatoes.com/" },
  { name: "Metacritic", url: "https://www.metacritic.com/" },
  { name: "Trakt", url: "https://trakt.tv/" },
  { name: "Letterboxd", url: "https://letterboxd.com/" },
  { name: "MyAnimeList", url: "https://myanimelist.net/" },
];

const ratingIcons = [
  { src: "/icons/imdb.webp", label: "IMDb", color: "rgb(180, 145, 15)" },
  { src: "/icons/tmdb.webp", label: "TMDB", color: "rgb(1, 155, 88)" },
  { src: "/icons/rt.webp", label: "Rotten Tomatoes", color: "rgb(185, 35, 8)" },
  { src: "/icons/rta.webp", label: "RT Audience", color: "rgb(185, 35, 8)" },
  { src: "/icons/mc.webp", label: "Metacritic", color: "rgb(75, 150, 38)" },
  { src: "/icons/trakt.webp", label: "Trakt", color: "rgb(175, 15, 45)" },
  { src: "/icons/lb.webp", label: "Letterboxd", color: "rgb(0, 155, 88)" },
  { src: "/icons/mal.webp", label: "MyAnimeList", color: "rgb(34, 60, 120)" },
];

const officialIcons = [
  { src: "/icons/official/imdb.webp", label: "IMDb" },
  { src: "/icons/official/tmdb.webp", label: "TMDB" },
  { src: "/icons/official/Rotten_Tomatoes_critic_positive.webp", label: "RT Fresh" },
  { src: "/icons/official/Rotten_Tomatoes_critic_certified_fresh.webp", label: "RT Certified Fresh" },
  { src: "/icons/official/Rotten_Tomatoes_critic_rotten.webp", label: "RT Rotten" },
  { src: "/icons/official/Rotten_Tomatoes_positive_audience.webp", label: "RT Audience Positive" },
  { src: "/icons/official/Rotten_Tomatoes_verified_hot_audience.webp", label: "RT Audience Verified Hot" },
  { src: "/icons/official/Rotten_Tomatoes_negative_audience.webp", label: "RT Audience Negative" },
  { src: "/icons/official/metacritic.webp", label: "Metacritic" },
  { src: "/icons/official/trakt.webp", label: "Trakt" },
  { src: "/icons/official/letterboxd.webp", label: "Letterboxd" },
  { src: "/icons/official/mal.webp", label: "MyAnimeList" },
];

const logos = [
  { src: "/examples/logo-nosferatu.webp", label: "Nosferatu (1922)" },
  { src: "/examples/logo-phantom.webp", alt: "The Phantom of the Opera (1925)", label: "Phantom of the Opera (1925)" },
  { src: "/examples/logo-trip-to-moon.webp", alt: "A Trip to the Moon (1902)", label: "A Trip to the Moon (1902)" },
  { src: "/examples/logo-safety-last.webp", alt: "Safety Last! (1923)", label: "Safety Last! (1923)" },
  { src: "/examples/logo-the-general.webp", label: "The General (1926)" },
];

const backdrops = [
  { src: "/examples/backdrop-nosferatu.webp", label: "Nosferatu (1922)" },
  { src: "/examples/backdrop-metropolis.webp", label: "Metropolis (1927)" },
  { src: "/examples/backdrop-caligari.webp", alt: "The Cabinet of Dr. Caligari (1920)", label: "Dr. Caligari (1920)" },
  { src: "/examples/backdrop-phantom.webp", alt: "The Phantom of the Opera (1925)", label: "Phantom of the Opera (1925)" },
  { src: "/examples/backdrop-trip-to-moon.webp", alt: "A Trip to the Moon (1902)", label: "A Trip to the Moon (1902)" },
  { src: "/examples/backdrop-safety-last.webp", alt: "Safety Last! (1923)", label: "Safety Last! (1923)" },
  { src: "/examples/backdrop-the-general.webp", label: "The General (1926)" },
  { src: "/examples/backdrop-namakura-gatana.webp", label: "Namakura Gatana (1917)" },
];

const posterSizes = [
  { src: "/examples/size-poster-xs.webp", label: "Extra Small" },
  { src: "/examples/size-poster-s.webp", label: "Small" },
  { src: "/examples/size-poster-m.webp", label: "Medium" },
  { src: "/examples/size-poster-l.webp", label: "Large" },
  { src: "/examples/size-poster-xl.webp", label: "Extra Large" },
];

const logoSizes = [
  { src: "/examples/size-logo-xs.webp", label: "Extra Small" },
  { src: "/examples/size-logo-s.webp", label: "Small" },
  { src: "/examples/size-logo-m.webp", label: "Medium" },
  { src: "/examples/size-logo-l.webp", label: "Large" },
  { src: "/examples/size-logo-xl.webp", label: "Extra Large" },
];

const backdropSizes = [
  { src: "/examples/size-backdrop-xs.webp", label: "Extra Small" },
  { src: "/examples/size-backdrop-s.webp", label: "Small" },
  { src: "/examples/size-backdrop-m.webp", label: "Medium" },
  { src: "/examples/size-backdrop-l.webp", label: "Large" },
  { src: "/examples/size-backdrop-xl.webp", label: "Extra Large" },
];

const episodes = [
  { src: "/examples/episode-bonanza.webp", label: "Standard" },
  { src: "/examples/episode-bonanza-h.webp", label: "Horizontal" },
  { src: "/examples/episode-bonanza-blur.webp", label: "Blurred (spoiler)" },
];


</script>

<template>
  <div class="min-h-screen flex flex-col">
    <main class="flex-1 flex flex-col items-center px-4 py-16">
      <div class="max-w-5xl w-full space-y-20">
        <!-- Hero -->
        <div class="text-center space-y-3">
          <h1 class="text-4xl font-bold tracking-tight sm:text-5xl">OpenPosterDB</h1>
          <p class="text-xs text-muted-foreground">v{{ version }}</p>
          <p class="text-lg text-muted-foreground max-w-xl mx-auto">
            Self-hosted poster, logo, and backdrop serving for your media server.
          </p>
          <div class="pt-4">
            <NavButtons />
          </div>
          <div class="pt-4 max-w-lg mx-auto">
            <FreeApiKeyCard />
          </div>
          <!-- Feature cards -->
          <div class="pt-4 grid gap-4 sm:grid-cols-2 lg:grid-cols-4 text-left">
            <Card v-for="f in features" :key="f.title">
              <CardHeader class="pb-0">
                <CardTitle class="text-sm flex items-center gap-2">
                  <component :is="f.icon" class="h-4 w-4 text-muted-foreground shrink-0" />
                  {{ f.title }}
                </CardTitle>
              </CardHeader>
              <CardContent>
                <p class="text-sm text-muted-foreground">{{ f.desc }}</p>
              </CardContent>
            </Card>
          </div>
        </div>

        <!-- Real poster examples -->
        <div class="space-y-4 text-center">
          <h2 class="text-2xl font-semibold">Rating Badges</h2>
          <p class="text-sm text-muted-foreground max-w-lg mx-auto">
            Aggregate ratings from IMDb, TMDB, Rotten Tomatoes, Metacritic, Trakt, Letterboxd, and
            MyAnimeList.
          </p>
          <p class="text-xs text-muted-foreground font-medium">Icon style</p>
          <div class="flex flex-wrap justify-center items-center gap-4" role="list" aria-label="Supported rating sources">
            <div v-for="icon in ratingIcons" :key="icon.src" role="listitem" class="flex flex-col items-center">
              <div
                class="h-10 w-10 rounded-lg p-1.5"
                :style="{ backgroundColor: icon.color }"
                :title="icon.label"
              >
                <BlurImage
                  :src="icon.src"
                  :alt="icon.label"
                  :width="28"
                  :height="28"
                  fit="contain"
                />
              </div>
            </div>
          </div>
          <p class="text-xs text-muted-foreground font-medium pt-2">Official style</p>
          <div class="flex flex-wrap justify-center items-center gap-4" role="list" aria-label="Official rating provider logos">
            <div v-for="icon in officialIcons" :key="icon.src" role="listitem" class="flex flex-col items-center">
              <div
                class="h-10 w-10 rounded-lg p-1.5 flex items-center justify-center bg-neutral-900"
                :title="icon.label"
              >
                <BlurImage
                  :src="icon.src"
                  :alt="icon.label"
                  :width="28"
                  :height="28"
                  fit="contain"
                />
              </div>
            </div>
          </div>
        </div>

        <!-- Posters -->
        <div class="space-y-4 text-center">
          <h2 class="text-2xl font-semibold">Posters</h2>
          <p class="text-sm text-muted-foreground max-w-lg mx-auto">
            Rating badges are overlaid directly onto your movie and TV posters.
          </p>
          <div class="flex flex-wrap justify-center gap-4">
            <div v-for="p in posters" :key="p.src" class="space-y-1">
              <BlurImage
                :src="p.src"
                :alt="p.alt || p.label"
                :width="160"
                :height="240"
                class="rounded-lg shadow-xl"
              />
              <p class="text-xs text-muted-foreground">{{ p.label }}</p>
            </div>
          </div>
        </div>

        <!-- Logos -->
        <div class="space-y-4 text-center">
          <h2 class="text-2xl font-semibold">Logos</h2>
          <p class="text-sm text-muted-foreground max-w-lg mx-auto">
            Serve transparent logos with rating badges attached below.
          </p>
          <div class="flex flex-wrap items-end justify-center gap-6">
            <div v-for="l in logos" :key="l.src" class="space-y-2">
              <div class="rounded-lg shadow-md bg-neutral-900 p-3">
                <BlurImage
                  :src="l.src"
                  :alt="l.alt || l.label"
                  :width="200"
                  :height="122"
                  fit="contain"
                />
              </div>
              <p class="text-xs text-muted-foreground">{{ l.label }}</p>
            </div>
          </div>
        </div>

        <!-- Backdrops -->
        <div class="space-y-4 text-center">
          <h2 class="text-2xl font-semibold">Backdrops</h2>
          <p class="text-sm text-muted-foreground max-w-lg mx-auto">
            Backdrops get rating badges in the top-right corner for a clean overlay.
          </p>
          <div class="flex flex-wrap justify-center gap-4">
            <div v-for="b in backdrops" :key="b.src" class="space-y-1">
              <BlurImage
                :src="b.src"
                :alt="b.alt || b.label"
                :width="360"
                :height="203"
                class="rounded-lg shadow-md"
              />
              <p class="text-xs text-muted-foreground">{{ b.label }}</p>
            </div>
          </div>
        </div>

        <!-- Episodes -->
        <div class="space-y-4 text-center">
          <h2 class="text-2xl font-semibold">Episodes</h2>
          <p class="text-sm text-muted-foreground max-w-lg mx-auto">
            Episode stills with rating badges and optional spoiler blur.
          </p>
          <div class="flex flex-wrap justify-center gap-4">
            <div v-for="e in episodes" :key="e.src" class="space-y-1">
              <BlurImage
                :src="e.src"
                :alt="e.label"
                :width="260"
                :height="146"
                class="rounded-lg shadow-md"
              />
              <p class="text-xs text-muted-foreground">{{ e.label }}</p>
            </div>
          </div>
        </div>

        <!-- Badge Position -->
        <div class="space-y-4 text-center">
          <h2 class="text-2xl font-semibold">Badge Position</h2>
          <p class="text-sm text-muted-foreground max-w-lg mx-auto">
            Place rating badges anywhere on the poster &mdash; corners, edges, or centered.
          </p>
          <div class="grid grid-cols-2 sm:grid-cols-4 gap-4 justify-items-center">
            <div v-for="p in positions" :key="p.src" class="space-y-2">
              <BlurImage
                :src="p.src"
                :alt="p.label"
                :width="160"
                :height="240"
                class="rounded-lg shadow-md"
              />
              <p class="text-xs text-muted-foreground">{{ p.label }}</p>
            </div>
          </div>
        </div>

        <!-- Badge Style -->
        <div class="space-y-4 text-center">
          <h2 class="text-2xl font-semibold">Badge Style</h2>
          <p class="text-sm text-muted-foreground max-w-lg mx-auto">
            Horizontal badges show the source icon and score side by side. Vertical badges stack
            them.
          </p>
          <div class="flex flex-wrap items-end justify-center gap-6">
            <div v-for="s in styles" :key="s.src" class="space-y-2">
              <BlurImage
                :src="s.src"
                :alt="s.label"
                :width="200"
                :height="300"
                class="rounded-lg shadow-md"
              />
              <p class="text-xs text-muted-foreground">{{ s.label }}</p>
            </div>
          </div>
        </div>

        <!-- Label Style -->
        <div class="space-y-4 text-center">
          <h2 class="text-2xl font-semibold">Label Style</h2>
          <p class="text-sm text-muted-foreground max-w-lg mx-auto">
            Show rating sources as colored icons, text labels, or official provider logos.
          </p>
          <div class="flex flex-wrap items-end justify-center gap-6">
            <div v-for="l in labels" :key="l.src" class="space-y-2">
              <BlurImage
                :src="l.src"
                :alt="l.label"
                :width="200"
                :height="300"
                class="rounded-lg shadow-md"
              />
              <p class="text-xs text-muted-foreground">{{ l.label }}</p>
            </div>
          </div>
        </div>

        <!-- Badge Size -->
        <div class="space-y-8 text-center">
          <div class="space-y-2">
            <h2 class="text-2xl font-semibold">Badge Size</h2>
            <p class="text-sm text-muted-foreground max-w-lg mx-auto">
              Control how large the rating badges appear on your images.
            </p>
          </div>

          <!-- Size: Posters -->
          <div class="space-y-2">
            <h3 class="text-lg font-medium">Posters</h3>
            <div class="flex flex-wrap justify-center gap-4">
              <div v-for="p in posterSizes" :key="p.src" class="space-y-1">
                <BlurImage
                  :src="p.src"
                  :alt="p.label"
                  :width="160"
                  :height="240"
                  class="rounded-lg shadow-md"
                />
                <p class="text-xs text-muted-foreground">{{ p.label }}</p>
              </div>
            </div>
          </div>

          <!-- Size: Logos -->
          <div class="space-y-2">
            <h3 class="text-lg font-medium">Logos</h3>
            <div class="flex flex-wrap items-end justify-center gap-6">
              <div v-for="l in logoSizes" :key="l.src" class="space-y-2">
                <div class="rounded-lg shadow-md bg-neutral-900 p-3">
                  <BlurImage
                    :src="l.src"
                    :alt="l.label"
                    :width="200"
                    :height="122"
                    fit="contain"
                  />
                </div>
                <p class="text-xs text-muted-foreground">{{ l.label }}</p>
              </div>
            </div>
          </div>

          <!-- Size: Backdrops -->
          <div class="space-y-2">
            <h3 class="text-lg font-medium">Backdrops</h3>
            <div class="flex flex-wrap justify-center gap-4">
              <div v-for="b in backdropSizes" :key="b.src" class="space-y-1">
                <BlurImage
                  :src="b.src"
                  :alt="b.label"
                  :width="360"
                  :height="203"
                  class="rounded-lg shadow-md"
                />
                <p class="text-xs text-muted-foreground">{{ b.label }}</p>
              </div>
            </div>
          </div>

        </div>

        <!-- Acknowledgments -->
        <div class="space-y-6 text-center">
          <h2 class="text-2xl font-semibold">Acknowledgments</h2>
          <p class="text-sm text-muted-foreground max-w-lg mx-auto">
            OpenPosterDB is made possible by these third-party services.
          </p>
          <div class="flex flex-wrap justify-center gap-3 text-left max-w-3xl mx-auto">
            <div
              v-for="p in dataProviders"
              :key="p.name"
              class="rounded-lg border p-3 w-full sm:w-[calc(50%-0.375rem)] lg:w-[calc(33.333%-0.5rem)]"
            >
              <p class="text-sm font-medium">{{ p.name }}</p>
              <p class="text-xs text-muted-foreground mb-2">{{ p.desc }}</p>
              <div class="flex gap-3">
                <a
                  :href="p.url"
                  target="_blank"
                  rel="noopener noreferrer"
                  class="text-xs text-muted-foreground hover:text-foreground transition-colors inline-flex items-center gap-1"
                >
                  <ExternalLink class="h-3 w-3" />
                  Homepage
                </a>
                <a
                  v-if="p.keyUrl"
                  :href="p.keyUrl"
                  target="_blank"
                  rel="noopener noreferrer"
                  class="text-xs text-muted-foreground hover:text-foreground transition-colors inline-flex items-center gap-1"
                >
                  <KeyRound class="h-3 w-3" />
                  Get API key
                </a>
              </div>
            </div>
          </div>
          <div class="pt-2">
            <p class="text-xs text-muted-foreground mb-2">Rating sources</p>
            <div class="flex flex-wrap justify-center gap-x-4 gap-y-1">
              <a
                v-for="r in ratingSources"
                :key="r.name"
                :href="r.url"
                target="_blank"
                rel="noopener noreferrer"
                class="text-xs text-muted-foreground hover:text-foreground transition-colors"
              >
                {{ r.name }}
              </a>
            </div>
          </div>
          <p class="text-xs text-muted-foreground italic max-w-lg mx-auto">
            This product uses the TMDB API but is not endorsed or certified by TMDB.
          </p>
        </div>

      </div>
    </main>
    <footer class="text-center py-6">
      <router-link to="/legal" class="text-xs text-muted-foreground hover:text-foreground transition-colors">
        Terms of Service
      </router-link>
    </footer>
  </div>
</template>
