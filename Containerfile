# Builder stage
FROM docker.io/clojure:tools-deps AS builder

RUN apt-get update && apt-get install -y curl build-essential && \
    curl https://sh.rustup.rs -sSf | sh -s -- -y && \
    . $HOME/.cargo/env && rustup install stable && rustup default stable

ENV PATH="/root/.cargo/bin:$PATH"

WORKDIR /app
COPY . .

ARG VERSION
ENV LIMABEAN_UBERJAR=/app/limabean-${VERSION}-standalone.jar

WORKDIR /app/rust
RUN cargo build --release

WORKDIR /app/clj
RUN clojure -T:build uber

# Runtime stage
FROM docker.io/clojure:tools-deps

ENV PATH="/app/bin:$PATH"
ENV LIMABEAN_BEANFILE=accounting.beancount

WORKDIR /app

COPY --from=builder /app/rust/target/release/limabean bin/
COPY --from=builder /app/rust/target/release/limabean-pod bin/
COPY --from=builder /app/clj/target/limabean-*-standalone.jar .

VOLUME /data
WORKDIR /data

ENTRYPOINT ["limabean"]
