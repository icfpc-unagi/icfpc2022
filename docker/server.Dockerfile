FROM golang:1.19.0 AS golang

FROM golang AS builder

RUN mkdir -p /work
WORKDIR /work
COPY go/go.mod go/go.sum /work/
RUN go mod download

COPY ./go/cmd /work/cmd
COPY ./go/pkg /work/pkg
COPY ./go/internal /work/internal
RUN go build -o /work/server ./cmd/server

FROM rust:1.63 AS rust-builder
RUN rustup target add x86_64-unknown-linux-musl
RUN rustup target add wasm32-unknown-unknown
# RUN cargo install wasm-pack  # It was very slow.
RUN curl https://rustwasm.github.io/wasm-pack/installer/init.sh -sSf | sh
RUN mkdir -p /work/.git /work/src
WORKDIR /work
COPY Cargo.lock /work/Cargo.lock
COPY Cargo.toml /work/Cargo.toml
RUN touch ./src/lib.rs && cargo vendor && cargo build --release && rm -rf ./src
COPY src/ /work/src/
RUN find /work/src -print -exec touch "{}" \; \
    && cargo build --release --bins \
    && wasm-pack build --release --target web

FROM golang AS tini
RUN wget -O /tini \
    https://github.com/krallin/tini/releases/download/v0.18.0/tini \
    && chmod +x /tini

FROM golang
ARG UNAGI_PASSWORD
WORKDIR /work
COPY --from=builder /work/server /usr/local/bin/server
RUN [ "${UNAGI_PASSWORD}" != "" ]
ENV SQL_ADDRESS 34.84.167.72
ENV SQL_USER root
ENV SQL_DATABASE database
ENV SQL_PASSWORD $UNAGI_PASSWORD
ENV UNAGI_PASSWORD $UNAGI_PASSWORD
COPY --from=tini /tini /tini
COPY --from=rust-builder /work/target/release/evaluate /usr/local/bin/evaluate
COPY --from=rust-builder /work/pkg /work/static/pkg
COPY ./static /work/static
COPY ./problems /work/problems
COPY ./secrets/login.json /work/secrets/login.json
ENTRYPOINT /tini -- /usr/local/bin/server --logtostderr
