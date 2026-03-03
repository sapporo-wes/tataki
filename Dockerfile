# ---- Build stage ----
FROM rust:bookworm AS builder

# liblzma-dev is required by lzma-sys (xz2 crate).
# perl is required by the vendored OpenSSL build.
# Other build essentials (gcc, make, pkg-config) are already in rust:bookworm.
RUN apt-get update \
    && apt-get install -y --no-install-recommends \
        perl \
        liblzma-dev \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /src
COPY . .
RUN cargo build --release

# ---- Runtime stage ----
FROM debian:bookworm-slim

LABEL org.opencontainers.image.authors="Tazro Inutano Ohta <inutano@gmail.com>"
LABEL org.opencontainers.image.url="https://github.com/sapporo-wes/tataki"
LABEL org.opencontainers.image.licenses="Apache-2.0"
LABEL org.opencontainers.image.description="CLI tool for detecting file formats in the bio-science field"

RUN apt-get update \
    && apt-get install -y --no-install-recommends \
        ca-certificates \
    && rm -rf /var/lib/apt/lists/*

COPY --from=builder /src/target/release/tataki /usr/bin/tataki

WORKDIR /work

ENTRYPOINT ["tataki"]
CMD ["--help"]
