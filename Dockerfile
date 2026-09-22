FROM debian:bookworm-slim

# No default value: an unset version must break the build instead of quietly
# installing whatever release happens to be tagged latest at build time.
ARG TATAKI_VERSION

# The OCI labels are set by docker/metadata-action in
# .github/workflows/build_release.yaml. Declaring any of them here as well would
# leave a dead LABEL line, because the labels the workflow passes to buildx win.

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
