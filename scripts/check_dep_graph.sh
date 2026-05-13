#!/usr/bin/env bash
# Enforce the dependency-direction rules for siderust-pod.
# Fails (exit 1) if any forbidden edge appears in a foundational crate's
# [dependencies] table.
#
# Lightweight implementation: greps Cargo.toml dependency lines.
# Not a full toml parser; intentional, to avoid extra build deps.

set -euo pipefail

cd "$(dirname "$0")/.."

fail=0

# Foundational upstream crates MUST NOT depend on siderust-pod.
for crate_dir in qtty tempoch affn cheby siderust; do
  toml="../$crate_dir/Cargo.toml"
  if [[ -f "$toml" ]]; then
    if grep -E "^\s*siderust-pod\b" "$toml" >/dev/null; then
      echo "FORBIDDEN edge: foundational crate $crate_dir depends on siderust-pod ($toml)"
      fail=1
    fi
  fi
done

if [[ "$fail" == "1" ]]; then
  echo
  echo "Dependency-graph check FAILED."
  exit 1
fi
echo "Dependency-graph check passed."
