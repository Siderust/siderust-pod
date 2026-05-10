# Dependency rules

```text
ALLOWED:
  qtty, tempoch, affn, cheby                  (independent foundations)
  siderust         -> qtty, tempoch, affn, cheby
  pod-core         -> siderust, qtty, tempoch, affn, cheby
  pod-dynamics     -> pod-core, siderust, cheby
  pod-io           -> pod-core
  pod-observations -> pod-core, pod-dynamics, siderust
  pod-estimation   -> pod-core, faer
  pod-qc           -> pod-core, pod-products
  pod-products     -> pod-core, pod-io
  pod-service      -> ALL pod-* crates
  pod-cli          -> pod-service (only)

FORBIDDEN:
  siderust          -> pod-*
  qtty/tempoch/affn/cheby -> siderust or pod-*
  pod-core          -> any pod-* crate
  pod-estimation    -> pod-io, pod-observations
  pod-observations  -> pod-io
  pod-dynamics      -> pod-io, pod-observations, pod-estimation
  pod-qc            -> pod-estimation
  pod-cli           -> pod-{core,dynamics,io,observations,estimation,qc,products}
```

The forbidden edges are checked by `scripts/check_dep_graph.sh`.
