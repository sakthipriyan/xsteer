<script setup>
import { onMounted, ref } from 'vue'

// Kit's confirmation email redirects a new subscriber to /?club=1. Without a note
// they land on a homepage that says nothing, which reads as a confirmation that
// did not work. Kit's redirect URL is the only thing that sets this parameter.
const PARAM = 'club'

const visible = ref(false)

onMounted(() => {
  const url = new URL(window.location.href)
  if (url.searchParams.get(PARAM) !== '1') return
  visible.value = true
  // Dropped on arrival, so a refresh or a copied link does not welcome someone who
  // never subscribed. Any other parameter and the hash survive.
  url.searchParams.delete(PARAM)
  history.replaceState(history.state, '', url.pathname + url.search + url.hash)
})
</script>

<template>
  <div v-if="visible" role="status" class="border-b bg-primary/10">
    <div class="mx-auto flex max-w-6xl items-center justify-between gap-4 px-6 py-3 text-sm">
      <p>
        <span class="font-semibold">You&rsquo;re in the Xsteer Club.</span>
        <span class="text-muted-foreground">
          Thanks for confirming &mdash; updates will reach your inbox as Xsteer takes shape.</span
        >
      </p>
      <button
        type="button"
        aria-label="Dismiss"
        title="Dismiss"
        class="inline-flex h-8 w-8 shrink-0 items-center justify-center rounded-lg text-muted-foreground transition-colors hover:bg-muted hover:text-foreground focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring focus-visible:ring-offset-2 focus-visible:ring-offset-background"
        @click="visible = false"
      >
        <svg
          class="h-4 w-4"
          viewBox="0 0 24 24"
          fill="none"
          stroke="currentColor"
          stroke-width="2"
          stroke-linecap="round"
          stroke-linejoin="round"
          aria-hidden="true"
        >
          <path d="M18 6 6 18M6 6l12 12" />
        </svg>
      </button>
    </div>
  </div>
</template>
