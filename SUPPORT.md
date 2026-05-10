# Support

## How to ask for help

* **Bugs / regressions:** open a GitHub issue with a minimal reproducer
  and attach the offending input file (after redacting anything
  sensitive).
* **Format compatibility issues** (SP3, RINEX, ANTEX, CRD, CPF, EOP):
  please attach the offending file or a redacted equivalent. We track
  parser-compatibility regressions as priority bugs.
* **Numerical / scientific questions:** open a discussion thread; the
  maintainers will route to the relevant module owner.
* **Security:** see [SECURITY.md](SECURITY.md). Do **not** report
  security issues via public issues.

## Commercial support

siderust-pod is licensed under AGPL-3.0-or-later. Commercial-licence
options and paid support contracts (response-time SLAs, prioritised
bug-fixes, embedded engineer hours) are being scoped as part of
milestone M12 — please contact `sales@siderust.org` for status.

## What is *not* supported

* Modifications to upstream foundational crates (`qtty`, `tempoch`,
  `affn`, `cheby`, `siderust`) — please direct those to the upstream
  repositories.
* Use of `siderust-pod-rest` on a public network in its current
  unauthenticated form.
