# Multi-stage Dockerfile producing a slim image with the POD CLI and REST server.
# Build:  docker build -t siderust-pod:dev -f Dockerfile ../..
#         (build context must include the workspaces for siderust-pod and the
#          read-only foundational crates: qtty, tempoch, affn, cheby, siderust)
# Run:    docker run --rm -p 8080:8080 siderust-pod:dev
FROM rust:1.82-bookworm AS builder
WORKDIR /workspace
COPY rust/ rust/
WORKDIR /workspace/rust/siderust-pod
RUN cargo build --release \
    -p siderust-pod-cli \
    -p siderust-pod-rest

FROM debian:bookworm-slim
RUN apt-get update \
 && apt-get install -y --no-install-recommends ca-certificates \
 && rm -rf /var/lib/apt/lists/*
COPY --from=builder /workspace/rust/siderust-pod/target/release/siderust-pod-cli  /usr/local/bin/siderust-pod
COPY --from=builder /workspace/rust/siderust-pod/target/release/siderust-pod-rest /usr/local/bin/siderust-pod-rest
ENV SIDERUST_POD_REST_BIND=0.0.0.0:8080
EXPOSE 8080
ENTRYPOINT ["siderust-pod-rest"]
