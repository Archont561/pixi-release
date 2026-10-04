# Run record — draft release visibility probe

Reproducibility record for the `doc-5` spike, including the two failed attempts. Conclusions
are in `doc-5`; the numbers are in `EVIDENCE.md`. Nothing here is normative.

**When.** 2026-10-04. **Where.** `Archont561/pixi-release`, branch
`arena/01a10687-pixi-release`.

## Subject

Draft release `v0.0.0-probe`, numeric id **`403008034`**, created with
`gh api --method POST /repos/Archont561/pixi-release/releases` and `draft: true`.

Asset upload was attempted 4× and failed every time with
`Post "https://uploads.github.com/…": EOF` — the sandbox cannot reach the upload host. The
probe proceeded without attached assets, which does not affect the visibility question: the
release object itself is what `contents: read` could or could not see.

No git tag was created by the draft (`git ls-remote --tags origin` empty throughout).

## Attempt 1 — commit `5ceb34c`, run `37202029536`

Three jobs at differing permission levels, each running the probes as ordinary steps.

**Outcome: all jobs `success`, and completely signal-free.** The steps swallowed their exit
codes, so `conclusion` reported nothing about what the probes found, and reading the logs was
impossible — `gh run view 37202029536 --log` failed with
`results-receiver.actions.githubusercontent.com … EOF`.

**Lesson, now in `EVIDENCE.md` §8:** encode findings in data that survives, not in pass/fail,
and never plan to read Actions logs from this sandbox.

## Attempt 2 — commit `28e7572`, run `37202120373`

Four jobs. Each probe captured stdout/stderr and exit status into a text fragment, and each job
appended its fragment to the **draft release body** over `api.github.com` — the one GitHub
surface this sandbox can both write and read.

All four jobs completed `success`. The verdict was then read with:

```
gh api repos/Archont561/pixi-release/releases/403008034 --jq .body
```

which returned the table now recorded in `EVIDENCE.md` §5.

## Cleanup — commit `00c47eb`

1. `gh release delete v0.0.0-probe` → **failed, `release not found`**. This is Finding 2
   reproducing itself during its own cleanup: there is no tag to resolve.
2. `gh api --method DELETE repos/Archont561/pixi-release/releases/403008034` → exit 0.
3. `git rm` the probe workflow; removed `.github/` entirely.
4. Verified: `gh release list` empty, `git ls-remote --tags origin` empty, zero `.github` files
   on the branch.

## Audit trail

`95336ff` initial commit → `5ceb34c` probe v1 → `28e7572` probe v2 → `00c47eb` probe removal.
The three probe commits are retained on the working branch deliberately; they are the only
remaining proof that the run happened.
