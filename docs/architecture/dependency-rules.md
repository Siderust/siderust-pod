# Dependency rules

```text
ALLOWED:
  qtty, tempoch, affn, cheby                  (independent foundations)
  siderust         -> qtty, tempoch, affn, cheby
  pod-core         -> siderust, qtty, tempoch, affn, cheby
  pod-dynamics     -> pod-core, siderust, cheby
  pod-io           -> siderust
  pod-observations -> pod-dynamics, siderust
  pod-estimation   -> siderust, faer
  pod-qc           -> pod-products, siderust
  pod-products     -> pod-io, siderust
  pod-service      -> ALL pod-* crates, siderust
  pod-cli          -> pod-service (only)

FORBIDDEN:
  siderust          -> pod-*
  qtty/tempoch/affn/cheby -> siderust or pod-*
  pod-core          -> any other pod-*
  pod-estimation    -> pod-io, pod-observations
  pod-observations  -> pod-io
  pod-dynamics      -> pod-io, pod-observations, pod-estimation
  pod-qc            -> pod-estimation
  pod-cli           -> pod-{core,dynamics,io,observations,estimation,qc,products}
```

The forbidden edges are checked by `scripts/check_dep_graph.sh`.
