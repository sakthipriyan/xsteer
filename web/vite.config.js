import { defineConfig } from 'vite'
import vue from '@vitejs/plugin-vue'
import { execSync } from 'node:child_process'
import { readFileSync } from 'node:fs'
import { fileURLToPath, URL } from 'node:url'

// Build provenance, baked in so a bug report can name the exact build it came
// from. Both values are best-effort: a tarball with no git history, or a
// checkout without Cargo.toml, must still produce a working site.

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

export default defineConfig({
  plugins: [vue()],
  // Honor PORT so the dev server can share a machine with other Vite projects.
  server: { port: Number(process.env.PORT) || 5173 },
  resolve: {
    alias: { '@': fileURLToPath(new URL('./src', import.meta.url)) },
  },
  define: {
    __COMMIT_HASH__: JSON.stringify(commitHash()),
    __APP_VERSION__: JSON.stringify(appVersion()),
  },
})
