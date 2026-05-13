#!/usr/bin/env bash
# Fails (exit 1) if any forbidden TODO/FIXME/XXX comment, `todo!()`, or
# `unimplemented!()` macro appears in non-doc source under crates/.
#
# Per the maturity plan (Phase 1.1) the source tree must end with zero of
# these markers. Doc-strings, fixture data, and format reference tables are
# excluded by limiting the scan to `*.rs` files and matching only on
# the macro/comment forms with leading whitespace.

set -euo pipefail

cd "$(dirname "$0")/.."

fail=0

scan() {
  local pattern="$1"
  local label="$2"
  # rg: search rust files only; strip doc-comments by ignoring '///' / '//!' lines
  if hits=$(grep -RInE "$pattern" \
              --include='*.rs' \
              --exclude-dir=target \
              crates/ 2>/dev/null \
            | grep -vE '^[^:]+:[0-9]+:\s*///' \
            | grep -vE '^[^:]+:[0-9]+:\s*//!' \
            | grep -vE '^[^:]+:[0-9]+:\s*\*'); then
    if [[ -n "$hits" ]]; then
      echo "FORBIDDEN ($label):"
      echo "$hits"
      echo
      fail=1
    fi
  fi
}

# `todo!(...)` macro
scan '\btodo!\s*\(' 'todo!() macro'
# `unimplemented!(...)` macro
scan '\bunimplemented!\s*\(' 'unimplemented!() macro'
# Bare `todo!` without args
scan '\btodo!\s*;' 'bare todo!;'
# `// TODO`, `// FIXME`, `// XXX` line comments (case-insensitive)
scan '//\s*(TODO|FIXME|XXX)\b' 'TODO/FIXME/XXX comment'

if [[ "$fail" == "1" ]]; then
  echo
  echo "TODO sweep FAILED."
  exit 1
fi
echo "TODO sweep passed."
