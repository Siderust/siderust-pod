# Security policy

## Reporting a vulnerability

Please report security vulnerabilities **privately** by emailing
`security@siderust.org` (or, if you do not have a corporate email
configured for the project, by opening a private security advisory on
the GitHub repository).

Please **do not** open public issues or pull requests for security
vulnerabilities. We will acknowledge your report within 5 working days
and will coordinate a disclosure timeline with you.

## Supported versions

Until siderust-pod reaches `1.0.0`, only the latest minor version on
`main` is supported.

## Scope

In scope:

* `siderust-pod-cli` / `siderust-pod-rest` binaries.
* Parsing of any externally-supplied file format (SP3, RINEX, ANTEX,
  CRD, CPF, EOP, configuration YAML).
* Any deserialisation entry point reachable from the REST surface.

Out of scope:

* Vulnerabilities in upstream foundational crates (`qtty`, `tempoch`,
  `affn`, `cheby`, `siderust`). Please report those upstream.

## Hardening status

The REST surface is currently *unauthenticated* (audit finding M-01).
Do not expose `siderust-pod-rest` to untrusted networks until milestone
M11 lands the bearer-token authentication and rate-limiting work.
