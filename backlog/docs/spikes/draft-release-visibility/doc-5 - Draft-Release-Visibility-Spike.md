---
id: doc-5
title: Draft Release Visibility Spike
type: other
created_date: '2026-10-04 12:32'
---
## Question

`doc-2` renders a `prove` job that is deliberately starved of permissions and asked to download
and execute the assets attached to a **draft** GitHub Release. The whole proof tier depends on
an unverified assumption:

> **Can a job with `permissions: contents: read` list and download draft release assets?**

Documentation is ambiguous. The cost of being wrong is a release pipeline that fails only on a
real release, so this was answered by experiment rather than by reading.

## Method

Run in `Archont561/pixi-release` on 2026-10-04, branch `arena/01a10687-pixi-release`.

Subject: a throwaway **draft** release `v0.0.0-probe`, numeric id **`403008034`**, created with
`gh api --method POST /repos/Archont561/pixi-release/releases` and `draft: true`. Asset upload
was attempted 4× and failed every time against `uploads.github.com`
(`.knowledge/research/agent-environment.md`), so the probes ran against the release object
itself — which is what the visibility question is about.

**Attempt 1 — commit `5ceb34c`, run `37202029536`. Discarded.** Three jobs at differing
permission levels ran the probes as ordinary steps. All jobs reported `success` and the run was
completely signal-free: the steps swallowed their exit codes, so `conclusion` said nothing about
what the probes found, and the logs were unreadable from this environment.

**Attempt 2 — commit `28e7572`, run `37202120373`. The measurement.** Four jobs, one per
permission set. Each probe captured stdout, stderr and exit status into a text fragment, and
each job appended its fragment to the **draft release body** over `api.github.com` — the one
GitHub surface this environment can both write and read. All four jobs completed. The verdict
was read with:

```
gh api repos/Archont561/pixi-release/releases/403008034 --jq .body
```

**Two method lessons.** Job and step `conclusion` are useless as probe signals once steps
swallow exit codes — encode findings in data that survives, not in pass/fail. And a probe must
report through a surface the observer can actually reach.

## Results

✅ Measured. `<tag>` is `v0.0.0-probe`; `{o}/{r}` is `Archont561/pixi-release`.

| Probe | `permissions: contents: read` | `permissions: contents: write` |
|---|---|---|
| `gh release list` | **ok — but empty** | ok, shows the draft |
| `gh release view <tag>` | **FAIL** `release not found` | ok |
| `gh release download <tag>` | **FAIL** `release not found` | ok |
| `GET /repos/{o}/{r}/releases/tags/<tag>` | **FAIL** `404 Not Found` | **FAIL** `404 Not Found` |
| `GET /repos/{o}/{r}/releases` | **ok — returns `[]`** | ok, includes the draft |

### Finding 1 — `contents: read` cannot see a draft, and fails *silently*

The answer to the question is **no**, and the failure mode is worse than a refusal: `gh release
list` exits 0 with no rows and the REST list returns `[]`. There is no 403 anywhere. A reader is
told the release does not exist, which points the investigation at the publish step that just
succeeded.

Corroboration: a public post-mortem in another project shipped exactly this bug — *"reading a
draft is the one read that needs write, and there is no narrower scope."*

### Finding 2 — get-by-tag 404s for a draft **even with write** (not anticipated)

A draft release **creates no git tag** — independently confirmed: `git ls-remote --tags` stayed
empty for the draft's whole lifetime. `GET /releases/tags/{tag}` therefore has nothing to
resolve. The `gh` CLI hides this by falling back to list-and-match on `tag_name`; the REST API
does not.

This bit the spike's own cleanup: `gh release delete v0.0.0-probe` returned `release not
found`, and the draft had to be deleted by numeric ID. GitHub had by then renamed the orphaned
draft to `untagged-5f8ddcdfb1feb84873c0`.

## Consequences for the design

Both findings changed `doc-2` rather than merely constraining it.

1. **`prove` runs with `permissions: {}` against the workflow artifacts**, not against the
   draft. Widening it to `contents: write` would have gutted the point of isolating it; moving
   the clean room to the artifacts gives a job holding **no repository token at all** — a
   stronger clean room than the design had before the spike.
2. **Upload integrity becomes a checksum read-back inside `stage`**, where `contents: write`
   already exists. It is a comparison, not an execution, so it never wanted a clean room.
3. **`release draft` emits a `release-id` output**; `triage`, `stage`, `publish` and `promote`
   address the draft by ID. Without this, every tag-addressed draft lookup in the design would
   have 404'd in production, because `pixi-release` is a Rust binary calling REST directly and
   gets no `gh` fallback.
4. **The forge port exposes `find_draft_by_tag_name()` as list + filter on `tag_name`**, never
   `GET /releases/tags/{tag}`, with a fake-backed regression test so it cannot come back.
5. **Evidence records store the release ID, not only the tag**, so `resume` still finds a draft
   GitHub has renamed to `untagged-<hex>`.

## Environment constraints that shaped the method

Not product knowledge, and recorded separately so the next spike does not rediscover them:
`.knowledge/research/agent-environment.md`.

## Disposal

1. `gh release delete v0.0.0-probe` → **failed, `release not found`** — Finding 2 reproducing
   itself during its own cleanup, since there is no tag to resolve. GitHub had by then renamed
   the orphaned draft to `untagged-5f8ddcdfb1feb84873c0`.
2. `gh api --method DELETE repos/Archont561/pixi-release/releases/403008034` → exit 0.
3. `git rm` the probe workflow; removed `.github/` entirely.
4. Verified: `gh release list` empty, `git ls-remote --tags origin` empty, zero `.github` files
   on the branch.

Commit trail, retained deliberately as the only remaining proof the run happened:
`95336ff` initial → `5ceb34c` probe v1 → `28e7572` probe v2 → `00c47eb` probe removal.
