# ADR-0004 — `cargo-public-api` snapshots gate intentional API changes

## Status

Accepted (Phase 1, CI advisory gate)

## Context

Pre-1.0 crates are allowed to break their API, but unintentional breakage
(caused by refactoring an internal impl and accidentally removing a public
re-export) is common and hard to catch in code review. We need a mechanism
that:

1. Makes API changes **visible** in PR diffs.
2. Fails CI when a change was unintentional (author forgot to update the snapshot).
3. Is easy to update when a change is intentional (run `--update` script).

## Decision

Each library crate under `crates/` maintains a committed
`crates/<crate>/api.snapshot` file containing the output of
`cargo public-api -p <crate> --simplified`.

The `scripts/snapshot_public_api.sh` script:
- Default mode: regenerates the snapshot in a temp location and `diff`s against
  committed; exits non-zero on any difference.
- `--update` mode: regenerates and overwrites all snapshots in-place.

CI runs the default mode in a nightly advisory job (`public-api` job in
`.github/workflows/ci.yml`). It is advisory (not blocking) during the 0.x
development phase; it will become mandatory at the 0.1.0 release milestone.

`cargo-public-api` requires nightly rustdoc JSON. The CI job uses the nightly
toolchain for this step only; the stable build/test matrix is unaffected.

## Consequences

- Every PR that changes a public API must also run `snapshot_public_api.sh --update`.
- The snapshot diffs are human-readable and appear in PR file-diffs, making
  accidental removals or renames immediately visible.
- Binary-only crates (no `[lib]`) are auto-skipped by the script.
- When the nightly toolchain changes the JSON format, spurious snapshot diffs
  may appear; regenerate with `--update` on the nightly bump PR.
