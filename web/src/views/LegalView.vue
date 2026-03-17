<script setup lang="ts">
import NavButtons from "@/components/NavButtons.vue";
</script>

<template>
  <div class="min-h-screen flex flex-col">
    <main class="flex-1 flex flex-col items-center px-4 py-16">
      <div class="max-w-2xl w-full space-y-3">
        <div class="space-y-3">
          <router-link
            to="/"
            class="text-sm text-muted-foreground hover:text-foreground transition-colors"
          >
            &larr; Back
          </router-link>
          <h1 class="text-3xl font-bold tracking-tight">Terms of Service</h1>
          <p class="text-sm text-muted-foreground">
            Last updated: March 17, 2026
          </p>
        </div>

        <section>
          <h2 class="text-lg font-semibold">Service Description</h2>
          <p class="text-sm text-muted-foreground">
            OpenPosterDB is an open-source service for generating and serving
            posters, logos, and backdrops with rating badge overlays for media
            servers. It can be self-hosted or accessed via the public instance
            at openposterdb.com. By using this service, whether self-hosted or
            through the public instance, you agree to the following terms.
          </p>
        </section>

        <section>
          <h2 class="text-lg font-semibold">Disclaimer</h2>
          <p class="text-sm text-muted-foreground">
            OpenPosterDB is provided "as is" without warranty of any kind,
            express or implied. Ratings displayed are sourced from third-party
            services and are not verified by OpenPosterDB. This project is not
            endorsed by or affiliated with any of the rating sources or data
            providers it integrates with.
          </p>
        </section>

        <section>
          <h2 class="text-lg font-semibold">Intellectual Property</h2>
          <p class="text-sm text-muted-foreground">
            All poster images, logos, backdrops, and metadata are sourced from
            third-party services (TMDB, Fanart.tv, etc.) and remain the property
            of their respective owners. Rating icons and trademarks belong to
            their respective owners (IMDb, Rotten Tomatoes, Metacritic,
            Letterboxd, Trakt, MyAnimeList, etc.). OpenPosterDB does not claim
            ownership of any third-party content.
          </p>
          <p class="text-sm text-muted-foreground">
            This product uses the TMDB API but is not endorsed or certified by
            TMDB.
          </p>
        </section>

        <section>
          <h2 class="text-lg font-semibold">User Responsibilities</h2>
          <p class="text-sm text-muted-foreground">
            Users are responsible for ensuring their use of OpenPosterDB
            complies with the terms of service of all integrated third-party
            APIs (TMDB, MDBList, OMDb, Fanart.tv). Users must obtain their own
            API keys where required and use them in accordance with each
            provider's terms.
          </p>
        </section>

        <section>
          <h2 class="text-lg font-semibold">Public Instance</h2>
          <p class="text-sm text-muted-foreground">
            The public instance at openposterdb.com is provided as a convenience
            and may be subject to rate limits, usage restrictions, or downtime
            at any time without notice. There is no guarantee of availability,
            uptime, or data retention. We reserve the right to suspend or
            terminate access to any account or API key on the public instance at
            our sole discretion.
          </p>
        </section>

        <section>
          <h2 class="text-lg font-semibold">Abuse</h2>
          <p class="text-sm text-muted-foreground">
            You may not use OpenPosterDB to engage in any activity that is
            abusive, or interferes with the operation of the service. Automated
            scraping, excessive API usage beyond reasonable limits, or
            redistribution of the service's output for commercial purposes
            without permission is prohibited.
          </p>
        </section>

        <section>
          <h2 class="text-lg font-semibold">Limitation of Liability</h2>
          <p class="text-sm text-muted-foreground">
            In no event shall OpenPosterDB or its contributors be liable for any
            direct, indirect, incidental, special, or consequential damages
            arising from the use or inability to use this service, including but
            not limited to loss of data or inaccurate ratings display.
          </p>
        </section>

        <section>
          <h2 class="text-lg font-semibold">Changes to Terms</h2>
          <p class="text-sm text-muted-foreground">
            These terms may be updated at any time. Continued use of the service
            constitutes acceptance of any changes.
          </p>
        </section>

        <div class="pt-4">
          <NavButtons />
        </div>
      </div>
    </main>
  </div>
</template>
