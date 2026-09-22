FROM debian:bookworm-slim

# No default value: an unset version must break the build instead of quietly
# installing whatever release happens to be tagged latest at build time.
ARG TATAKI_VERSION


LABEL org.opencontainers.image.authors="Tazro Ohta (tazro.ohta@chiba-u.jp)"
LABEL org.opencontainers.image.url="https://github.com/sapporo-wes/tataki"
LABEL org.opencontainers.image.version="v0.3.0"
LABEL org.opencontainers.image.licenses="Apache2.0"
LABEL org.opencontainers.image.description="CLI tool designed primarily for detecting file formats in the bio-science field"

RUN apt-get update && apt-get install -y --fix-missing --no-install-recommends\
    curl \
    ca-certificates \
    && apt-get clean && rm -rf /tmp/* /var/tmp/* \
    && rm -rf /var/lib/apt/lists/*

RUN curl -fsSL -o /tmp/docker.tgz https://download.docker.com/linux/static/stable/$(uname -m)/docker-24.0.9.tgz && \
    tar -C /tmp -xf /tmp/docker.tgz && \
    mv /tmp/docker/* /usr/bin/ && \
    rm -rf /tmp/docker /tmp/docker.tgz

# The binary is pinned to the tag being built rather than fetched from
# releases/latest, so that rebuilding an old tag yields the binary that tag
# shipped instead of the newest one.
RUN curl -fsSL -o /usr/bin/tataki https://github.com/sapporo-wes/tataki/releases/download/${TATAKI_VERSION}/tataki-$(uname -m) && \
    chmod +x /usr/bin/tataki

WORKDIR /app

ENTRYPOINT [ "tataki" ]
CMD [ "--help" ]
