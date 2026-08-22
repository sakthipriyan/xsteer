// Emits robots.txt and _headers into dist/ after the Vite build.
//
// Production and beta serve the same code, so the only safe place to differentiate
// crawler policy is the build. beta.xsteer.in must never be indexed — a staging copy
// competing with the real site in search is a hard problem to undo later.
//
// The default is deliberately the *restrictive* one: an unset or unrecognized
// DEPLOY_ENV produces a noindex build. Failing closed means a misconfigured workflow
// costs a deploy, not the domain's search presence.

import { writeFileSync } from 'node:fs'
import { join } from 'node:path'

const DIST = join(import.meta.dirname, '..', 'dist')
const env = process.env.DEPLOY_ENV ?? 'beta'
const isProduction = env === 'production'

// Beta deliberately ALLOWS crawling. Disallow and noindex are not additive — they
// conflict. A disallowed crawler never fetches the page, so it never sees the
// X-Robots-Tag below, and the bare URL can still surface in results when something
// links to it. Letting the crawler in is what makes noindex binding.
//
// It also sidesteps Cloudflare's Managed robots.txt, which prepends its own
// `User-agent: * / Allow: /` block to whatever we serve. Two conflicting groups for
// the same agent resolve differently across crawlers; agreeing with it removes the
// ambiguity entirely.
const robots = isProduction
  ? `User-agent: *
Allow: /

Sitemap: https://xsteer.in/sitemap.xml
`
  : `# Crawling is allowed on purpose. Every response on this host carries
# X-Robots-Tag: noindex, nofollow — see web/scripts/seo.mjs.
User-agent: *
Allow: /
`

// _headers is honored by Cloudflare Workers static assets, same syntax as Pages.
const headers = `/*
  X-Content-Type-Options: nosniff
  Referrer-Policy: strict-origin-when-cross-origin
  X-Frame-Options: DENY
${isProduction ? '' : '  X-Robots-Tag: noindex, nofollow\n'}
/assets/*
  Cache-Control: public, max-age=31536000, immutable
`

writeFileSync(join(DIST, 'robots.txt'), robots)
writeFileSync(join(DIST, '_headers'), headers)

if (isProduction) {
  writeFileSync(
    join(DIST, 'sitemap.xml'),
    `<?xml version="1.0" encoding="UTF-8"?>
<urlset xmlns="http://www.sitemaps.org/schemas/sitemap/0.9">
  <url><loc>https://xsteer.in/</loc></url>
</urlset>
`,
  )
}

console.log(`seo: DEPLOY_ENV=${env} → ${isProduction ? 'indexable' : 'noindex'}`)
