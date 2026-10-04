#!/usr/bin/env bash
# TEMPORARY diagnostic harness. Delete with diagnose.yml.
set -u
report=/tmp/report.md
: > "$report"
{
  echo "## diagnose \`$(git rev-parse --short HEAD)\`"
  echo
} >> "$report"

run() {
  local name="$1"; shift
  local out status
  out="$("$@" 2>&1)"; status=$?
  {
    echo "<details><summary><code>${name}</code> → exit ${status}</summary>"
    echo
    echo '```'
    printf '%s\n' "$out" | tail -c 12000
    echo '```'
    echo
    echo "</details>"
    echo
  } >> "$report"
}

run "bun-install"     pixi run bun-install
run "fmt-check-js"    pixi run -- bun x biome check .
run "cargo-fmt"       pixi run -- cargo fmt --manifest-path Cargo.toml --all --check
run "cargo-clippy"    pixi run -- cargo clippy --manifest-path Cargo.toml --workspace --all-targets --all-features -- -D warnings
run "typecheck"       pixi run typecheck
run "test-root"       pixi run -- bun test scripts
run "cargo-test"      pixi run -- cargo test --manifest-path Cargo.toml --workspace
run "lint-actions"    pixi run lint-actions
run "repo-check"      pixi run repo-check
run "docs-build"      pixi run -- bash -c "cd apps/docs && bun run scripts/build.ts"
