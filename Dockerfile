FROM debian:bookworm-slim


LABEL org.opencontainers.image.authors="Tazro Ohta (tazro.ohta@chiba-u.jp)"
LABEL org.opencontainers.image.url="https://github.com/sapporo-wes/tataki"
LABEL org.opencontainers.image.version="v0.2.2"
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

# ADD https://github.com/sapporo-wes/tataki/releases/latest/download/tataki /usr/bin/tataki
RUN curl -fsSL -o /usr/bin/tataki https://github.com/sapporo-wes/tataki/releases/latest/download/tataki-$(uname -m) && \
    chmod +x /usr/bin/tataki

WORKDIR /app

ENTRYPOINT [ "tataki" ]
CMD [ "--help" ]

