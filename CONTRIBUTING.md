# Contributing

Thanks for contributing to `siderust-pod`.

## Development layout

The project currently uses sibling path dependencies from the Siderust stack. A development checkout should look like:

```text
siderust-stack/
  affn/
  cheby/
  qtty/
  siderust/
  siderust-pod/
  tempoch/
```

All of these repositories are available under the [Siderust GitHub organisation](https://github.com/Siderust).

## Before opening a pull request

Run:

```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo test --workspace --no-fail-fast
cargo test --workspace --no-default-features --no-fail-fast
cargo test --workspace --all-features --no-fail-fast
cargo doc --workspace --no-deps
bash scripts/check_dep_graph.sh
bash scripts/check_no_todos.sh
```

## Scientific and numerical changes

Changes to orbital dynamics, estimation, reference-frame transformations, time handling, or file-format semantics should include:

- a clear statement of assumptions and validity range;
- tests against an independent reference, published example, or frozen fixture where practical;
- units and reference frames made explicit at API boundaries;
- a note in the pull request explaining any expected numerical tolerance.

## API changes

This project is pre-1.0, so APIs can evolve. Still, prefer small, reviewable changes and document user-visible behaviour changes.

## Security

Do not disclose vulnerabilities in public issues or pull requests. Follow [SECURITY.md](SECURITY.md).
