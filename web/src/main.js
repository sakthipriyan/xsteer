import { createApp } from 'vue'
import './style.css'
import App from './App.vue'

// The dark class is set by the pre-paint script in index.html and owned from there
// on by ThemeToggle.vue; doing it here would be one deferred frame too late.
createApp(App).mount('#app')
