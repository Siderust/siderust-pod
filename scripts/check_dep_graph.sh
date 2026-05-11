#!/usr/bin/env bash
# Enforce the dependency-direction rules from docs/architecture/dependency-rules.md.
# Fails (exit 1) if any forbidden edge appears in a crate's [dependencies] table.
#
# Lightweight implementation: greps Cargo.toml dependency lines per crate.
# Not a full toml parser; intentional, to avoid extra build deps.

set -euo pipefail

cd "$(dirname "$0")/.."

forbidden() {
  # Args: crate name, forbidden dep substring
  local crate="$1"
  local dep="$2"
  local toml="crates/$crate/Cargo.toml"
  if [[ ! -f "$toml" ]]; then
    return 0
  fi
  if grep -E "^\s*${dep}\b" "$toml" >/dev/null; then
    echo "FORBIDDEN edge: $crate -> $dep ($toml)"
    return 1
  fi
  return 0
}

fail=0

# pod-core MUST NOT depend on any other pod-* crate
for d in dynamics io observations estimation qc products service cli rest; do
  forbidden siderust-pod-core "siderust-pod-$d" || fail=1
done

# pod-estimation MUST NOT depend on pod-io / pod-observations
for d in io observations; do
  forbidden siderust-pod-estimation "siderust-pod-$d" || fail=1
done

# pod-observations MUST NOT depend on pod-io
forbidden siderust-pod-observations siderust-pod-io || fail=1

# pod-dynamics MUST NOT depend on pod-io / pod-observations / pod-estimation
for d in io observations estimation; do
  forbidden siderust-pod-dynamics "siderust-pod-$d" || fail=1
done

# pod-qc MUST NOT depend on pod-estimation
forbidden siderust-pod-qc siderust-pod-estimation || fail=1

# pod-cli MUST NOT depend on the compute crates directly
for d in core dynamics io observations estimation qc products; do
  forbidden siderust-pod-cli "siderust-pod-$d" || fail=1
done

# Reusable foundational crates MUST NOT depend on any siderust-pod-* crate.
# Enforced for the canonical foundational crates that may be present as
# sibling checkouts under ../<crate>.
for crate_dir in qtty tempoch affn cheby siderust; do
  toml="../$crate_dir/Cargo.toml"
  if [[ -f "$toml" ]]; then
    if grep -E "^\s*siderust-pod-[a-z]+\b" "$toml" >/dev/null; then
      echo "FORBIDDEN edge: foundational crate $crate_dir depends on a siderust-pod-* crate ($toml)"
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
