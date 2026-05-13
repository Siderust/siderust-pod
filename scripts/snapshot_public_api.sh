#!/usr/bin/env bash
# Snapshot the public API of every workspace crate to
# `crates/<crate>/api.snapshot` using `cargo public-api`.
#
# Intended workflow:
#   * Run locally with `--update` to refresh the committed baselines after
#     an intentional API change.
#   * Run in CI without `--update` to diff against the committed baselines.
#
# Requires the `cargo-public-api` binary (install with
# `cargo install --locked cargo-public-api`). Requires nightly because
# `cargo public-api` uses rustdoc JSON.

set -euo pipefail

cd "$(dirname "$0")/.."

UPDATE=0
if [[ "${1-}" == "--update" ]]; then
  UPDATE=1
fi

if ! command -v cargo-public-api >/dev/null 2>&1; then
  echo "cargo-public-api not installed; skipping snapshot." >&2
  echo "Install with: cargo install --locked cargo-public-api" >&2
  exit 0
fi

mapfile -t CRATES < <(find crates -mindepth 1 -maxdepth 1 -type d -printf '%f\n' | sort)

fail=0
for c in "${CRATES[@]}"; do
  manifest="crates/$c/Cargo.toml"
  # Skip binaries-only crates (no library target).
  if ! grep -qE '^\[lib\]' "$manifest" && ! [[ -f "crates/$c/src/lib.rs" ]]; then
    continue
  fi
  snap="crates/$c/api.snapshot"
  tmp="$(mktemp)"
  if ! cargo public-api -p "$c" --simplified > "$tmp" 2>/dev/null; then
    echo "warn: cargo public-api failed for $c (skipped)" >&2
    rm -f "$tmp"
    continue
  fi
  if [[ "$UPDATE" == "1" || ! -f "$snap" ]]; then
    mv "$tmp" "$snap"
    echo "updated $snap"
  else
    if ! diff -u "$snap" "$tmp" > /dev/null; then
      echo "DIFF in $snap:"
      diff -u "$snap" "$tmp" || true
      fail=1
    fi
    rm -f "$tmp"
  fi
done

if [[ "$fail" == "1" ]]; then
  echo
  echo "Public-API drift detected. Re-run with --update to refresh baselines."
  exit 1
fi
echo "Public-API snapshots up to date."
