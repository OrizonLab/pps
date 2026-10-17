# syntax=docker/dockerfile:1
#
# Image d'un service Rust de PPS. Le binaire à construire est choisi avec
# l'argument BINAIRE (par exemple : --build-arg BINAIRE=pps-api).
#
# Étapes : préparation des dépendances (cargo-chef, pour le cache), compilation,
# puis copie du seul binaire dans une image distroless sans shell, non root.
# Les images de base sont épinglées par empreinte.

# Debian 13, même glibc que l'image d'exécution. Le tag permet à Dependabot
# de suivre la bonne variante ; l'empreinte fige l'image exacte.
FROM rust:1.99.0-trixie@sha256:81015672d6939b0265f09adf96d5de125482bb1330106076956b0518e74edc90 AS chef
RUN cargo install cargo-chef --locked --version 0.1.78
WORKDIR /app

FROM chef AS plan
COPY . .
RUN cargo chef prepare --recipe-path recipe.json

FROM chef AS compilation
ARG BINAIRE
COPY --from=plan /app/recipe.json recipe.json
RUN cargo chef cook --release --locked --recipe-path recipe.json
COPY . .
ENV SQLX_OFFLINE=true
RUN cargo build --release --locked --bin "${BINAIRE}" \
    && cp "target/release/${BINAIRE}" /app/service

# Aucune faille haute ou critique connue au 2026-10-17.
FROM gcr.io/distroless/cc-debian13:nonroot@sha256:e792ab3d241a468a4fd7519ddbbebe66b49b5f365771716ea688ad40b6c6f1c2
COPY --from=compilation /app/service /usr/local/bin/service
USER nonroot:nonroot
ENTRYPOINT ["/usr/local/bin/service"]
