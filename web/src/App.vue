<script setup>
import { computed } from 'vue'
import PlanPreview from './components/PlanPreview.vue'
import ThemeToggle from './components/ThemeToggle.vue'

// dev and beta serve the same build as production, so the host is the only thing
// that tells them apart at runtime. Without a badge, dev.xsteer.in is
// indistinguishable from the real site — which is how a preview gets mistaken
// for what is actually live.
const environment = computed(() => {
  if (typeof window === 'undefined') return null
  const host = window.location.hostname
  if (host.startsWith('dev.')) return 'dev'
  if (host.startsWith('beta.')) return 'beta'
  return null
})

// Baked in at build time by vite.config.js. Empty when git, Cargo.toml or the
// registry was not readable, in which case that entry simply does not render.
const commit = __COMMIT_HASH__
const version = __APP_VERSION__

// A commit is only worth showing where builds move faster than versions do. On
// dev, on beta and on a local dev server, `version` is whatever Cargo.toml last
// said — not a tag — so the SHA is the only thing that names the build. On
// production the version *is* the tag and the SHA adds nothing.
//
// The hosts that show it are listed rather than inferred from "not production",
// so a domain this file has not heard of fails closed instead of putting a SHA
// on the live site. It hangs off Xsteer alone: the engines ship as released
// crates, and a crates.io version is already exact.
const showCommit = computed(() => {
  if (!commit || typeof window === 'undefined') return false
  if (environment.value !== null) return true
  const host = window.location.hostname
  return host === 'localhost' || host === '127.0.0.1' || host.endsWith('.localhost')
})

// Every repository the product is assembled from, in the order the Open source
// section names them. A parse or planning bug belongs to one of the engines
// rather than to Xsteer, so a report is only actionable with their versions too.
//
// Each part links to what it names: the repository, the tag that version was cut
// at, the commit itself. All three repositories tag `v<version>` — the format
// `cargo xtask release` writes — so the tag URL is derivable rather than stored.
const products = computed(() =>
  [
    { name: 'Xsteer', version, commit: showCommit.value ? commit : null },
    { name: 'Xfingine', version: __XFINGINE_VERSION__, commit: null },
    { name: 'Xfina', version: __XFINA_VERSION__, commit: null },
  ]
    .filter((p) => p.version)
    .map((p) => {
      const repoUrl = `https://github.com/sakthipriyan/${p.name.toLowerCase()}`
      return {
        ...p,
        repoUrl,
        versionUrl: `${repoUrl}/tree/v${p.version}`,
        // The dirty marker is for the reader, not for GitHub.
        commitUrl: p.commit ? `${repoUrl}/commit/${p.commit.replace('*', '')}` : null,
      }
    }),
)

const steps = [
  {
    n: '01',
    title: 'Upload your statements',
    body: 'Bank accounts, credit cards, mutual fund CAS, IBKR activity. Parsed in your browser by Xfina — the files never leave your device.',
  },
  {
    n: '02',
    title: 'Give each account a job',
    body: 'Salary, spends, medical, travel, investment. Each carries a cash buffer, the cards it pays, and the fixed expenses it covers.',
  },
  {
    n: '03',
    title: 'Get the month planned',
    body: 'Xsteer reconciles balances against dues and buffers, then emits an ordered, dated list of exactly what to move where.',
  },
  {
    n: '04',
    title: 'Execute and reconcile',
    body: 'Tick steps off as you do them. Next month’s statement confirms what actually happened against what was planned.',
  },
]

const reads = [
  { kind: 'Bank accounts', items: 'Axis, Bank of Baroda, HDFC, ICICI, State Bank of India' },
  { kind: 'Credit cards', items: 'Axis, HDFC, ICICI' },
  { kind: 'Mutual funds', items: 'CAMS consolidated statement' },
  { kind: 'International brokerage', items: 'Interactive Brokers activity statements' },
]

// The three repositories the product is assembled from, ordered from the reader
// outwards: the thing they came here for first, then the layers underneath it.
const projects = [
  {
    name: 'Xsteer',
    tagline: 'Open source personal finance OS',
    body: 'This project. Turns a parsed vault into an ordered, dated plan for the month, and gives you the interface to execute it against.',
    href: 'https://github.com/sakthipriyan/xsteer',
  },
  {
    name: 'Xfingine',
    tagline: 'Open source financial engine',
    body: 'Pure computation engines for personal finance planning — inflation-adjusted EMI schedules and more. A Rust core, published to crates.io, npm and PyPI.',
    href: 'https://github.com/sakthipriyan/xfingine',
  },
  {
    name: 'Xfina',
    tagline: 'Open source data parser',
    body: 'Reads Indian bank, credit card, mutual fund and brokerage statements into structured data. Rust compiled to WebAssembly, so parsing happens on your machine.',
    href: 'https://github.com/sakthipriyan/xfina',
  },
]
</script>

<template>
  <div class="min-h-screen">
    <!-- Nav -->
    <header class="sticky top-0 z-50 border-b bg-background/80 backdrop-blur">
      <div class="mx-auto flex max-w-6xl items-center justify-between px-6 py-4">
        <a href="/" class="flex items-center gap-2.5">
          <img src="/favicon.svg" alt="" class="h-7 w-7" />
          <span class="text-lg font-semibold tracking-tight">Xsteer</span>
          <span
            v-if="environment"
            class="rounded-full border border-primary/40 px-2 py-0.5 text-[11px] font-medium uppercase tracking-wide text-primary"
          >
            {{ environment }}
          </span>
        </a>
        <nav class="flex items-center gap-6 text-sm text-muted-foreground">
          <a href="#how" class="hidden transition-colors hover:text-foreground sm:inline">How it works</a>
          <a href="#privacy" class="hidden transition-colors hover:text-foreground sm:inline"
            >Privacy First</a
          >
          <a href="#open-source" class="transition-colors hover:text-foreground">Open Source</a>
          <ThemeToggle />
        </nav>
      </div>
    </header>

    <!-- Hero -->
    <section class="relative overflow-hidden">
      <div
        class="pointer-events-none absolute inset-x-0 -top-40 h-80 bg-gradient-to-b from-primary/10 to-transparent blur-3xl"
        aria-hidden="true"
      />
      <div class="mx-auto max-w-6xl px-6 pb-16 pt-20 sm:pt-28">
        <p class="mb-5 text-sm font-medium uppercase tracking-widest text-primary">
          In development
        </p>
        <h1 class="max-w-3xl text-4xl font-bold leading-[1.1] tracking-tight sm:text-6xl">
          Your statements in.<br />
          A month&rsquo;s money to-do list out.
        </h1>
        <p class="mt-6 max-w-2xl text-lg leading-relaxed text-muted-foreground">
          You already know roughly what to do with your salary each month. Xsteer works out
          the exact amounts &mdash; what to move between accounts, what each credit card
          needs before its due date, and what is genuinely left to invest.
        </p>

        <!-- The flow -->
        <div class="mt-10 flex flex-wrap items-center gap-x-3 gap-y-2 font-mono text-sm">
          <span class="rounded-md bg-muted px-3 py-1.5 font-medium">Salary</span>
          <span class="text-muted-foreground">&rarr;</span>
          <span class="rounded-md bg-muted px-3 py-1.5 font-medium">Expenses</span>
          <span class="text-muted-foreground">&rarr;</span>
          <span class="rounded-md bg-muted px-3 py-1.5 font-medium">Card payment</span>
          <span class="text-muted-foreground">&rarr;</span>
          <span class="rounded-md bg-muted px-3 py-1.5 font-medium">Surplus</span>
          <span class="text-muted-foreground">&rarr;</span>
          <span class="rounded-md bg-primary/10 px-3 py-1.5 font-medium text-primary">Splits</span>
        </div>

        <div class="mt-10 flex flex-wrap gap-3">
          <a
            href="https://github.com/sakthipriyan/xsteer"
            rel="noopener"
            class="rounded-lg bg-primary px-5 py-2.5 text-sm font-semibold text-primary-foreground transition-opacity hover:opacity-90"
          >
            Follow on GitHub
          </a>
          <a
            href="https://github.com/sakthipriyan/xsteer/blob/main/docs/DESIGN.md"
            rel="noopener"
            class="rounded-lg border px-5 py-2.5 text-sm font-semibold transition-colors hover:bg-muted"
          >
            Read the design
          </a>
        </div>
      </div>
    </section>

    <!-- The output -->
    <section class="mx-auto max-w-6xl px-6 py-16">
      <div class="mb-8 max-w-2xl">
        <h2 class="text-2xl font-semibold tracking-tight sm:text-3xl">
          The whole product is this list
        </h2>
        <p class="mt-3 text-muted-foreground">
          Not a dashboard to interpret. An ordered, dated set of instructions, with the
          reason each one exists.
        </p>
      </div>
      <PlanPreview />
    </section>

    <!-- How it works -->
    <section id="how" class="scroll-mt-3 border-t bg-muted/30">
      <div class="mx-auto max-w-6xl px-6 py-16">
        <h2 class="text-2xl font-semibold tracking-tight sm:text-3xl">How it works</h2>
        <div class="mt-10 grid gap-8 sm:grid-cols-2">
          <div v-for="s in steps" :key="s.n" class="flex gap-5">
            <span class="font-mono text-sm font-semibold text-primary">{{ s.n }}</span>
            <div>
              <h3 class="font-semibold">{{ s.title }}</h3>
              <p class="mt-2 text-sm leading-relaxed text-muted-foreground">{{ s.body }}</p>
            </div>
          </div>
        </div>
      </div>
    </section>

    <!-- Privacy -->
    <section id="privacy" class="mx-auto max-w-6xl scroll-mt-3 px-6 py-16">
      <div class="max-w-2xl">
        <h2 class="text-2xl font-semibold tracking-tight sm:text-3xl">Privacy first</h2>
        <p class="mt-3 leading-relaxed text-muted-foreground">
          Your statements are parsed, your plan computed and your vault stored without any
          of it ever leaving your device.
        </p>
      </div>
      <div class="mt-12 grid gap-10 lg:grid-cols-2 lg:gap-16">
        <div>
          <h3 class="text-xl font-semibold tracking-tight sm:text-2xl">
            There is no server to trust
          </h3>
          <p class="mt-4 leading-relaxed text-muted-foreground">
            <span class="font-medium text-foreground"
              >Statement parsing and every calculation run as WebAssembly inside your own
              browser.</span
            > Your vault is encrypted at rest on your device, with a key the browser will
            not hand back out through its Web Crypto API. The app itself opens without a
            passphrase &mdash; a passphrase guards the backup you export.
          </p>
          <p class="mt-4 leading-relaxed text-muted-foreground">
            That cuts both ways, and it is worth saying plainly:
            <span class="font-medium text-foreground"
              >browser storage is disposable.</span
            >
            Clearing site data wipes the vault, and the backup you exported is the only
            copy that survives it. There is no restore link, because there is nobody on
            the other end to restore it.
          </p>
        </div>
        <div>
          <h3 class="text-xl font-semibold tracking-tight sm:text-2xl">What it reads locally</h3>
          <dl class="mt-6 space-y-4">
            <div v-for="r in reads" :key="r.kind" class="border-b pb-4 last:border-0">
              <dt class="text-sm font-semibold">{{ r.kind }}</dt>
              <dd class="mt-1 text-sm text-muted-foreground">{{ r.items }}</dd>
            </div>
          </dl>
        </div>
      </div>
    </section>

    <!-- Open source -->
    <section id="open-source" class="scroll-mt-3 border-t bg-muted/30">
      <div class="mx-auto max-w-6xl px-6 py-16">
        <div class="max-w-2xl">
          <h2 class="text-2xl font-semibold tracking-tight sm:text-3xl">Open source</h2>
          <p class="mt-3 leading-relaxed text-muted-foreground">
            Software that decides what to do with your salary should be software you can
            read. Every layer &mdash; the planner, the computation engines beneath it, and
            the parser that reads your statements &mdash; is a separate Apache-2.0
            repository you can audit, fork, or run yourself.
          </p>
        </div>
        <div class="mt-10 grid gap-6 sm:grid-cols-3">
          <a
            v-for="p in projects"
            :key="p.name"
            :href="p.href"
            rel="noopener"
            class="group rounded-lg border bg-background p-6 transition-colors hover:border-primary/40"
          >
            <h3 class="font-semibold tracking-tight transition-colors group-hover:text-primary">
              {{ p.name }}
            </h3>
            <p class="mt-1 text-sm font-medium text-primary">{{ p.tagline }}</p>
            <p class="mt-3 text-sm leading-relaxed text-muted-foreground">{{ p.body }}</p>
          </a>
        </div>
      </div>
    </section>

    <!-- Footer -->
    <footer class="border-t">
      <div
        class="mx-auto flex max-w-6xl flex-col gap-4 px-6 py-10 text-sm text-muted-foreground sm:flex-row sm:items-end sm:justify-between"
      >
        <p>Xsteer is being built in the open. Nothing here is investment advice.</p>
        <!-- Build provenance, deliberately quiet: it is for a bug report, not a reader. -->
        <p v-if="products.length" class="font-mono text-xs sm:text-right">
          <template v-for="(p, i) in products" :key="p.name">
            <span v-if="i" aria-hidden="true"> · </span>
            <span class="whitespace-nowrap">
              <a
                :href="p.repoUrl"
                rel="noopener"
                class="transition-colors hover:text-foreground"
                >{{ p.name }}</a
              >{{ ' '
              }}<a
                :href="p.versionUrl"
                rel="noopener"
                class="transition-colors hover:text-foreground"
                >{{ p.version }}</a
              ></span
            >
            <span v-if="p.commit" class="whitespace-nowrap">
              (<a
                :href="p.commitUrl"
                rel="noopener"
                class="transition-colors hover:text-foreground"
                >{{ p.commit }}</a
              >)</span
            >
          </template>
        </p>
      </div>
    </footer>
  </div>
</template>
