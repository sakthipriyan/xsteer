# Deploying xsteer.in

| | Host | Deploys when |
|---|---|---|
| **Production** | `xsteer.in`, `www.xsteer.in` | a `v*.*.*` tag is pushed, or a manual run |
| **Beta** | `beta.xsteer.in` | `cargo xtask beta` on any branch, and every push to `main` |

Both serve the same code from an assets-only Cloudflare Worker. The builds differ in
exactly one respect: `DEPLOY_ENV=beta` bakes in `noindex` and a `Disallow: /` robots.txt,
so the beta site can never compete with the real one in search.

`web/scripts/seo.mjs` **defaults to noindex** when `DEPLOY_ENV` is unset. A broken
workflow costs a deploy, never the domain's search presence.

---

## The release flow

Work happens on a branch, is previewed on beta, then merged and tagged. Beta serves
whatever branch was last deployed to it — not `main` — so a change is seen in a real
deploy *before* it lands.

```bash
git switch -c feat/thing        # work; tests run on every push
cargo xtask beta                # push the branch and deploy it to beta.xsteer.in
cargo xtask prepare-release minor   # bump the version, open a CHANGELOG section
cargo xtask beta                # preview the release commit itself

git rebase main
git checkout main
git merge --ff-only feat/thing  # fast-forward, so the tested commit *is* the main commit
cargo xtask release             # tag it; production deploys
```

**The merge must be a fast-forward.** A squash creates a new commit, so the SHA beta
validated would no longer exist on `main` — the tag would point at code that, in that
exact form, was never deployed anywhere. Rebase first and the fast-forward is always
available.

### What `cargo xtask release` refuses

| Refuses when | Because |
|---|---|
| not on `main`, or the tree is dirty | a release must be a commit that exists |
| `main` ≠ `origin/main` | tagging a stale local `main` produces a release nobody can find |
| the tag already exists, locally or on origin | versions are not reusable |
| no Deploy Beta run succeeded for this exact SHA | beta is only a gate if promotion checks it |

The tag name comes from `[workspace.package] version` in `Cargo.toml` and is never
typed, so the two cannot drift. `--skip-beta-check` exists for the case where you know
why the run is missing; it is a deliberate override, not a fallback.

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

Production goes out on a tag, which `cargo xtask release` creates from the workspace
version:

```bash
cargo xtask release
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
