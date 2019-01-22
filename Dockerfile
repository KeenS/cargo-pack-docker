FROM rust:1.32.0-stretch
ENV CARGO_PACK_DOKCER_VERSION 0.3.3
ENV DOCKER_VERSION 18.03.1-ce
RUN wget -q https://github.com/KeenS/cargo-pack-docker/releases/download/v${CARGO_PACK_DOKCER_VERSION}/cargo-pack-docker-v${CARGO_PACK_DOKCER_VERSION}-x86_64-unknown-linux-gnu.tar.gz && \
        tar xzf cargo-pack-docker-v${CARGO_PACK_DOKCER_VERSION}-x86_64-unknown-linux-gnu.tar.gz && \
        install cargo-pack-docker /usr/local/bin && \
        rm -rf cargo-pack-docker cargo-pack-docker-v${CARGO_PACK_DOKCER_VERSION}-x86_64-unknown-linux-gnu.tar.gz
