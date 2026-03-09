import { createApp } from 'vue'
import { createPinia } from 'pinia'
import { VueQueryPlugin } from '@tanstack/vue-query'

import App from './App.vue'
import router from './router'
import { setOnAuthFailure } from './lib/api'
import './assets/main.css'

const app = createApp(App)

app.use(createPinia())
app.use(router)

setOnAuthFailure(() => router.push({ name: 'login' }))
app.use(VueQueryPlugin)

app.mount('#app')
