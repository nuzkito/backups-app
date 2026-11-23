# Exact version, for reproducible builds. To update Rust, change the tag and run `make up`.
FROM rust:1.100.0-slim-trixie

ARG UID=1000
ARG GID=1000
ARG CARGO_AUDIT_VERSION=0.23.0

# Cross-compiler to build Windows executables.
# mesa-vulkan-drivers (lavapipe) allows rendering the screenshots without a GPU.
# zip: builds the portable Windows package (`make portable`).
RUN apt-get update \
    && apt-get install -y --no-install-recommends gcc-mingw-w64-x86-64 zip mesa-vulkan-drivers \
    && rm -rf /var/lib/apt/lists/* \
    && rustup target add x86_64-pc-windows-gnu \
    && rustup component add clippy rustfmt

# cargo-audit: known vulnerabilities in the dependencies (`make audit`).
# The base image makes /usr/local/cargo writable by all users, but `cargo install` creates its
# cache files as root. Without them, cargo creates them again as the dev user.
RUN cargo install --locked cargo-audit@${CARGO_AUDIT_VERSION} \
    && rm -rf /usr/local/cargo/registry/* /usr/local/cargo/.package-cache* /usr/local/cargo/.global-cache

# User with the same UID/GID as the host user.
# registry: `cargo install` creates it as root, and the named volume copies this owner.
RUN groupadd --gid ${GID} dev \
    && useradd --uid ${UID} --gid ${GID} --create-home dev \
    && chown ${UID}:${GID} /usr/local/cargo/registry

USER dev
WORKDIR /app
