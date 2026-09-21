#!/usr/bin/env bash
# Fail if implementation source contains TODO/FIXME/XXX comments or
# todo!/unimplemented! macros. Documentation comments are excluded.

set -euo pipefail

cd "$(dirname "$0")/.."

roots=(src)
[[ -d examples ]] && roots+=(examples)

fail=0

scan() {
  local pattern="$1"
  local label="$2"

  if hits=$(grep -RInE "$pattern" \
              --include='*.rs' \
              --exclude-dir=target \
              "${roots[@]}" 2>/dev/null \
            | grep -vE '^[^:]+:[0-9]+:[[:space:]]*///' \
            | grep -vE '^[^:]+:[0-9]+:[[:space:]]*//!' \
            | grep -vE '^[^:]+:[0-9]+:[[:space:]]*\*'); then
    if [[ -n "$hits" ]]; then
      echo "FORBIDDEN ($label):"
      echo "$hits"
      echo
      fail=1
    fi
  fi
}

scan '\btodo!\s*\(' 'todo!() macro'
scan '\bunimplemented!\s*\(' 'unimplemented!() macro'
scan '\btodo!\s*;' 'bare todo!;'
scan '//\s*(TODO|FIXME|XXX)\b' 'TODO/FIXME/XXX comment'

if [[ "$fail" == "1" ]]; then
  echo
  echo "TODO sweep FAILED."
  exit 1
fi

echo "TODO sweep passed."
