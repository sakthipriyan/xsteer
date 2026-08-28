<script setup>
import { onBeforeUnmount, onMounted, ref } from 'vue'

// The key is only written once the visitor makes a choice. Its absence is meaningful:
// it means "follow the OS", which is what lets a machine switching to night mode carry
// the page with it. Kept in step with the pre-paint bootstrap in index.html.
const STORAGE_KEY = 'xsteer-theme'

const isDark = ref(false)
let media = null

// localStorage throws outright in some privacy configurations. A theme preference is
// not worth breaking the page over, so every access degrades to "no stored choice".
function readStored() {
  try {
    return localStorage.getItem(STORAGE_KEY)
  } catch {
    return null
  }
}

function store(value) {
  try {
    localStorage.setItem(STORAGE_KEY, value)
  } catch {
    /* preference lasts for this page view only */
  }
}

function apply(dark) {
  isDark.value = dark
  document.documentElement.classList.toggle('dark', dark)
}

function toggle() {
  apply(!isDark.value)
  store(isDark.value ? 'dark' : 'light')
}

function onSystemChange(event) {
  if (!readStored()) apply(event.matches)
}

onMounted(() => {
  // index.html has already set the class; read it back rather than recomputing, so the
  // button can never disagree with what is on screen.
  isDark.value = document.documentElement.classList.contains('dark')
  media = window.matchMedia('(prefers-color-scheme: dark)')
  media.addEventListener('change', onSystemChange)
})

onBeforeUnmount(() => media?.removeEventListener('change', onSystemChange))
</script>

<template>
  <button
    type="button"
    :aria-pressed="isDark"
    :title="isDark ? 'Switch to light' : 'Switch to dark'"
    :aria-label="isDark ? 'Switch to light theme' : 'Switch to dark theme'"
    class="inline-flex h-9 w-9 items-center justify-center rounded-lg border transition-colors hover:bg-muted hover:text-foreground focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring focus-visible:ring-offset-2 focus-visible:ring-offset-background"
    @click="toggle"
  >
    <!-- The icon shows the destination, matching the title: sun means "go light". -->
    <svg
      v-if="isDark"
      class="h-[1.15rem] w-[1.15rem]"
      viewBox="0 0 24 24"
      fill="none"
      stroke="currentColor"
      stroke-width="2"
      stroke-linecap="round"
      stroke-linejoin="round"
      aria-hidden="true"
    >
      <circle cx="12" cy="12" r="4" />
      <path d="M12 2v2M12 20v2M4.93 4.93l1.41 1.41M17.66 17.66l1.41 1.41M2 12h2M20 12h2M6.34 17.66l-1.41 1.41M19.07 4.93l-1.41 1.41" />
    </svg>
    <svg
      v-else
      class="h-[1.15rem] w-[1.15rem]"
      viewBox="0 0 24 24"
      fill="none"
      stroke="currentColor"
      stroke-width="2"
      stroke-linecap="round"
      stroke-linejoin="round"
      aria-hidden="true"
    >
      <path d="M12 3a6 6 0 0 0 9 9 9 9 0 1 1-9-9Z" />
    </svg>
  </button>
</template>
