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

Run in `Archont561/pixi-release` on 2026-10-04, on branch `arena/01a10687-pixi-release`.

1. Created a throwaway **draft** release `v0.0.0-probe` (release ID `403008034`) via
   `gh api --method POST /repos/{o}/{r}/releases` with `draft: true`.
2. Pushed a temporary probe workflow with four jobs. Each job ran the same five probes under a
   different permission set, and **every probe swallowed its exit code** and recorded the
   outcome as text.
3. Jobs reported findings by **appending to the draft release body over `api.github.com`**, not
   as workflow artifacts or log output — see *Constraints* below.
4. Read the verdict with `gh api /repos/{o}/{r}/releases/403008034 --jq .body`.
5. Deleted the draft, the workflow and the probe commits' effects; verified `gh release list`
   empty and `git ls-remote --tags origin` empty.

Run `37202120373`. The raw transcript is `.knowledge/research/DRAFT-RELEASE-PROBE.md`.

**Two method notes that mattered.** First, job and step `conclusion` are useless as probe
signals once steps swallow exit codes — findings must be encoded in data that survives, not in
pass/fail. Second, the first attempt (run `37202029536`) reported all jobs `success` while being
entirely signal-free; it had to be redesigned.

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

## Constraints discovered about the execution environment

⚠️ Environment-specific, recorded so the next spike does not rediscover them: the agent sandbox
reaches `api.github.com` but **not** GitHub's blob/upload hosts. `uploads.github.com` (release
asset upload) and `results-receiver.actions.githubusercontent.com` (`gh run view --log`) both
fail with `EOF`. A probe must therefore report through an `api.github.com`-readable surface —
release body, PR comment, or check output — and never rely on reading Actions logs.

## Disposal

The draft release, the probe workflow and `.github/` were all removed; `gh release list` and
`git ls-remote --tags origin` are both empty. Three probe commits (`5ceb34c`, `28e7572`,
`00c47eb`) remain on the working branch as an audit trail.
