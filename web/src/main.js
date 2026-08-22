import { createApp } from 'vue'
import './style.css'
import App from './App.vue'

// Respect the OS preference on first paint; a toggle can override it later.
if (window.matchMedia('(prefers-color-scheme: dark)').matches) {
  document.documentElement.classList.add('dark')
}

createApp(App).mount('#app')
