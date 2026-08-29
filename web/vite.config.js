import { defineConfig } from 'vite'
import vue from '@vitejs/plugin-vue'
import { execSync } from 'node:child_process'
import { readFileSync } from 'node:fs'
import { fileURLToPath, URL } from 'node:url'

// Build provenance, baked in so a bug report can name the exact build it came
// from. Every value here is best-effort: a tarball with no git history, a
// checkout without Cargo.toml, or an unreachable registry must still produce a
// working site.

// CI hands us the SHA directly; locally we ask git, and mark a dirty tree so a
// screenshot of uncommitted work is never mistaken for a real build.
function commitHash() {
  if (process.env.GITHUB_SHA) return process.env.GITHUB_SHA.slice(0, 7)
  try {
    const sha = execSync('git rev-parse --short HEAD', { stdio: ['ignore', 'pipe', 'ignore'] })
      .toString()
      .trim()
    const dirty = execSync('git status --porcelain', { stdio: ['ignore', 'pipe', 'ignore'] })
      .toString()
      .trim().length
    return dirty ? `${sha}*` : sha
  } catch {
    return ''
  }
}

// The workspace Cargo.toml is the single source of truth for the version — the
// same line `cargo xtask release` derives its tag from. Reading it here keeps
// the site from becoming a third place that has to be remembered.
function appVersion() {
  try {
    const cargo = readFileSync(fileURLToPath(new URL('../Cargo.toml', import.meta.url)), 'utf8')
    const table = cargo.split('[workspace.package]')[1] ?? ''
    return table.match(/^version = "([^"]+)"/m)?.[1] ?? ''
  } catch {
    return ''
  }
}

// Cargo.lock, not Cargo.toml: the manifest carries a requirement (`"0.2"`), and a
// bug report needs the version that was actually compiled in (`0.2.4`). A crate
// that is not a dependency yet resolves to ''.
function lockedVersion(crate) {
  try {
    const lock = readFileSync(fileURLToPath(new URL('../Cargo.lock', import.meta.url)), 'utf8')
    return lock.match(new RegExp(`^name = "${crate}"\\nversion = "([^"]+)"`, 'm'))?.[1] ?? ''
  } catch {
    return ''
  }
}

// The fallback for a crate the footer names but the build does not yet depend
// on: ask crates.io what is published. Deliberately second to `lockedVersion` —
// the day Xfingine becomes a real dependency, the version compiled in is the
// truthful one and takes over on its own. An unreachable registry must never
// fail a deploy, so a timeout or an error just drops the entry from the line.
async function publishedVersion(crate) {
  try {
    const res = await fetch(`https://crates.io/api/v1/crates/${crate}`, {
      headers: { 'User-Agent': 'xsteer-web build (https://github.com/sakthipriyan/xsteer)' },
      signal: AbortSignal.timeout(5000),
    })
    if (!res.ok) return ''
    return (await res.json())?.crate?.max_stable_version ?? ''
  } catch {
    return ''
  }
}

export default defineConfig(async () => ({
  plugins: [vue()],
  // Honor PORT so the dev server can share a machine with other Vite projects.
  server: { port: Number(process.env.PORT) || 5173 },
  resolve: {
    alias: { '@': fileURLToPath(new URL('./src', import.meta.url)) },
  },
  define: {
    __COMMIT_HASH__: JSON.stringify(commitHash()),
    __APP_VERSION__: JSON.stringify(appVersion()),
    __XFINGINE_VERSION__: JSON.stringify(
      lockedVersion('xfingine') || (await publishedVersion('xfingine')),
    ),
    __XFINA_VERSION__: JSON.stringify(lockedVersion('xfina')),
  },
}))
