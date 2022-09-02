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

FROM golang AS tini
RUN wget -O /tini \
        https://github.com/krallin/tini/releases/download/v0.18.0/tini \
    && chmod +x /tini

FROM golang
ARG UNAGI_PASSWORD
COPY --from=builder /work/server /usr/local/bin/server
RUN [ "${UNAGI_PASSWORD}" != "" ]
ENV SQL_ADDRESS 34.84.167.72
ENV SQL_USER root
ENV SQL_DATABASE database
ENV SQL_PASSWORD $UNAGI_PASSWORD
COPY --from=tini /tini /tini
ENTRYPOINT /tini -- /usr/local/bin/server --logtostderr
