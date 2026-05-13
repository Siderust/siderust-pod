# Architecture Overview — A Diagrammatic Tour

This document gives a two-page tour of the `siderust-pod` workspace: the
crate graph, the data flow through a precise-orbit-determination (POD) run,
and how the workspace composes with its read-only upstream crates.

For the prose-heavy companion documents see
[`boundaries.md`](./boundaries.md), [`dependency-rules.md`](./dependency-rules.md),
[`providers.md`](./providers.md) and [`dispatch.md`](./dispatch.md).

---

## Crate Graph

The workspace contains 15 crates organised in four tiers:

1. **Upstream foundations** (`qtty`, `tempoch`, `affn`, `cheby`, `siderust`) —
   read-only, vendored at the workspace root and consumed via
   `[patch.crates-io]` redirections.
2. **Sibling primitive crates** (`siderust-{dynamics,lambert,sgp4,spice,tle}`) —
   reusable astronomy/astrodynamics building blocks that do not depend on any
   POD-specific concept.
3. **POD layer** (`siderust-pod-{core,dynamics,io,observations,estimation,qc,products}`) —
   POD-specific composition of the primitives.
4. **Application layer** (`siderust-pod-service`, `siderust-pod-cli`,
   `siderust-pod-rest`) — orchestration, CLI binary, and HTTP service.

```mermaid
flowchart TD
    %% Upstream foundations
    subgraph Upstream["Upstream (read-only, [patch.crates-io])"]
        QTTY[qtty]
        TEMPOCH[tempoch]
        AFFN[affn]
        CHEBY[cheby]
        SIDERUST[siderust]
    end

    %% Sibling primitives
    subgraph Siblings["Sibling primitives"]
        DYN[siderust-dynamics]
        LAMB[siderust-lambert]
        SGP4[siderust-sgp4]
        SPICE[siderust-spice]
        TLE[siderust-tle]
    end

    %% POD layer
    subgraph PodLayer["POD layer"]
        CORE[siderust-pod-core]
        IO[siderust-pod-io]
        PDYN[siderust-pod-dynamics]
        OBS[siderust-pod-observations]
        EST[siderust-pod-estimation]
        QC[siderust-pod-qc]
        PROD[siderust-pod-products]
    end

    %% Application layer
    subgraph App["Application layer"]
        SVC[siderust-pod-service]
        CLI[siderust-pod-cli]
        REST[siderust-pod-rest]
    end

    %% Upstream edges
    QTTY --> AFFN
    TEMPOCH --> AFFN
    QTTY --> SIDERUST
    TEMPOCH --> SIDERUST
    AFFN --> SIDERUST
    CHEBY --> SIDERUST

    %% Sibling edges
    SIDERUST --> DYN
    SIDERUST --> LAMB
    QTTY --> SGP4
    TEMPOCH --> SGP4
    AFFN --> SGP4
    TLE --> SGP4
    CHEBY --> SPICE
    AFFN --> SPICE
    TEMPOCH --> SPICE
    QTTY --> TLE

    %% POD core depends on upstream + siderust
    QTTY --> CORE
    TEMPOCH --> CORE
    AFFN --> CORE
    SIDERUST --> CORE

    %% POD layer
    CORE --> IO
    SGP4 --> IO
    SPICE --> IO
    TLE --> IO

    CORE --> PDYN
    DYN --> PDYN
    SIDERUST --> PDYN

    CORE --> OBS
    IO --> OBS

    CORE --> EST
    CORE --> QC
    IO --> QC

    CORE --> PROD
    IO --> PROD

    %% Service composes everything
    CORE --> SVC
    IO --> SVC
    PDYN --> SVC
    OBS --> SVC
    EST --> SVC
    QC --> SVC
    PROD --> SVC
    DYN --> SVC
    SPICE --> SVC

    %% Apps
    SVC --> CLI
    SVC --> REST
```

The acyclicity of this graph is enforced by tooling (see
[`dependency-rules.md`](./dependency-rules.md)).

---

## Data Flow

A POD run is a deterministic, manifest-tracked transformation from raw
observation files to scientific products. The 15-stage pipeline is described
in detail in `siderust-pod-service`; the user-visible flow is:

```mermaid
flowchart LR
    RAW[("Raw inputs<br/>SP3 · RINEX OBS/NAV<br/>ANTEX · EOP · CRD · CPF · TLE")]
    IO["siderust-pod-io<br/>(parsers)"]
    OBS["siderust-pod-observations<br/>(measurement models<br/>+ corrections)"]
    DYN["siderust-pod-dynamics<br/>(force models<br/>+ STM)"]
    EST["siderust-pod-estimation<br/>(WLS / EKF)"]
    QC["siderust-pod-qc<br/>(residual stats<br/>orbit comparison)"]
    PROD["siderust-pod-products<br/>(SP3 · OEM · manifest)"]
    OUT[("Run artefacts<br/>+ canonical manifest")]

    RAW --> IO
    IO --> OBS
    IO --> DYN
    DYN --> EST
    OBS --> EST
    EST --> QC
    EST --> PROD
    QC --> PROD
    PROD --> OUT
```

`siderust-pod-service` orchestrates this flow; `siderust-pod-cli` and
`siderust-pod-rest` are thin frontends that hand a `RunConfig` to the service
and stream results back.

---

## Composition with Upstream

The POD workspace does **not** vendor or fork its foundational crates. It
consumes them as path overrides in the workspace `Cargo.toml`:

```toml
[patch.crates-io]
qtty-core    = { path = "../qtty/qtty-core" }
tempoch-core = { path = "../tempoch/tempoch-core" }
qtty         = { path = "../qtty/qtty" }
tempoch      = { path = "../tempoch/tempoch" }
affn         = { path = "../affn" }
siderust     = { path = "../siderust" }
```

This means:

- The five upstream trees (`qtty`, `tempoch`, `affn`, `cheby`, `siderust`)
  are **read-only** from the perspective of this workspace. Behavioural
  changes that belong upstream must be made in those trees first and then
  pulled in.
- POD never adds astronomy-domain concepts to upstream crates and never adds
  POD-specific assumptions to the primitives. The
  [separation-of-concerns rules](./boundaries.md) define which layer owns
  which concept.
- All POD crates pin their upstream deps through the workspace
  `[patch.crates-io]` section; individual crate `Cargo.toml` files refer to
  the regular crates-io names so the workspace can be built without the path
  overrides if a downstream user vendors a published version.

The POD layer is the *only* place where these primitives are composed into
end-to-end orbit-determination workflows; siblings such as `siderust-sgp4`
and `siderust-spice` remain pure primitive providers.
