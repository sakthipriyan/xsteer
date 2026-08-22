# Deploying xsteer.in

| | Host | Deploys when |
|---|---|---|
| **Production** | `xsteer.in`, `www.xsteer.in` | a `v*.*.*` tag is pushed, or a manual run |
| **Beta** | `beta.xsteer.in` | every push to `main` |

Both serve the same code from an assets-only Cloudflare Worker. The builds differ in
exactly one respect: `DEPLOY_ENV=beta` bakes in `noindex` and a `Disallow: /` robots.txt,
so the beta site can never compete with the real one in search.

`web/scripts/seo.mjs` **defaults to noindex** when `DEPLOY_ENV` is unset. A broken
workflow costs a deploy, never the domain's search presence.

---

## Scripted setup

Steps 1–2 and 5 can be done from the terminal instead of two dashboards —
`scripts/setup-dns.sh` drives the Cloudflare API and Spaceship's public API.
Read-only first:

```bash
./scripts/setup-dns.sh status
```

Credentials come from the environment and are never echoed. Keep them outside the
repo:

```bash
set -a; source ~/.xsteer-env; set +a
./scripts/setup-dns.sh apply
./scripts/setup-dns.sh wait
```

Two things the script cannot do for you: creating the Cloudflare API token and
creating the Spaceship API key both require their respective dashboards once.

Note that zone creation needs a **different, broader token** than CI uses — the Workers
token in step 3 below cannot create zones. Make a temporary one with `Zone:Zone:Edit`
and `Zone:DNS:Edit`, then delete it when setup is done.

---

## One-time setup

Steps 1–4 are manual and can only be done by the account owner.

### 1. Add the domain to Cloudflare

Create a Cloudflare account if you do not have one, then **Add a domain** → `xsteer.in`
→ **Free** plan.

### 2. Move the nameservers at Spaceship

`xsteer.in` currently uses Spaceship's nameservers:

```text
launch1.spaceship.net
launch2.spaceship.net
```

Cloudflare's free plan cannot run alongside them — CNAME-only (partial) setup is a
Business-plan feature. So the domain has to move to the two nameservers Cloudflare
assigns you.

In the Spaceship dashboard: **Domains → xsteer.in → Nameservers → Custom**, and replace
both with Cloudflare's pair. Propagation is usually under an hour.

> `xfina.dev` uses the same Spaceship nameservers but is a separate zone — moving
> `xsteer.in` does not affect it.

Verify:

```bash
dig +short NS xsteer.in
```

### 3. Create a Cloudflare API token

**My Profile → API Tokens → Create Token → "Edit Cloudflare Workers"** template.

Then confirm the token grants all of the following, adding any the template omits:

| Scope | Permission | Why |
|---|---|---|
| Account | Workers Scripts → Edit | deploy the Worker |
| Zone (`xsteer.in`) | Workers Routes → Edit | bind the custom domains |
| Zone (`xsteer.in`) | DNS → Edit | `custom_domain` routes create DNS records |

Copy the token once — Cloudflare will not show it again. Your **Account ID** is on the
right-hand side of any Cloudflare dashboard page.

### 4. Add the GitHub secrets

```bash
gh secret set CLOUDFLARE_API_TOKEN  --repo sakthipriyan/xsteer
gh secret set CLOUDFLARE_ACCOUNT_ID --repo sakthipriyan/xsteer
```

### 5. First deploy

Push to `main` and the **Deploy Beta** workflow runs. Wrangler creates the
`beta.xsteer.in` custom domain and its DNS record on first deploy — there is nothing to
add in the Cloudflare DNS tab by hand.

Production goes out on a tag:

```bash
git tag v0.1.0 && git push origin v0.1.0
```

---

## Optional: redirect www to the apex

Both `xsteer.in` and `www.xsteer.in` are bound to the production Worker, and the page
declares `xsteer.in` as canonical, so search engines will not treat them as duplicates.
If you would rather `www` redirect outright, add a Cloudflare **Redirect Rule** (Rules →
Redirect Rules, free on any plan): match hostname `www.xsteer.in`, 301 to
`https://xsteer.in${uri.path}`, then drop the `www` route from `wrangler.jsonc`.

---

## Deploying by hand

Rarely needed, but useful when debugging the pipeline. Requires `wrangler login` or
`CLOUDFLARE_API_TOKEN` in your shell.

```bash
npm install          # installs wrangler at the repo root
npm run deploy:beta
npm run deploy:production
```

---

## Rolling back

Cloudflare keeps every deployed version:

```bash
npx wrangler deployments list --name xsteer
npx wrangler rollback --name xsteer
```

For beta, use `--name xsteer-beta`.

---

## Troubleshooting

**Custom domain stuck on "Initializing"** — the zone's nameservers have not finished
moving. Re-check step 2; wrangler cannot create a route on a zone Cloudflare does not
yet serve.

**`Authentication error [code: 10000]`** — the token is missing one of the three scopes
in step 3. DNS → Edit is the one the Workers template most often leaves out.

**Beta is showing up in Google** — check the deploy log for
`seo: DEPLOY_ENV=beta → noindex`, then confirm `curl -sI https://beta.xsteer.in`
returns `x-robots-tag: noindex, nofollow`.
