FROM rust:1.63 AS rust-builder

RUN apt-get update -qy && apt-get install -qy apt-transport-https ca-certificates gnupg curl
RUN echo "deb [signed-by=/usr/share/keyrings/cloud.google.gpg] http://packages.cloud.google.com/apt cloud-sdk main" \
    | tee -a /etc/apt/sources.list.d/google-cloud-sdk.list \
    && curl https://packages.cloud.google.com/apt/doc/apt-key.gpg \
    | apt-key --keyring /usr/share/keyrings/cloud.google.gpg  add - \
    && apt-get update -y && apt-get install google-cloud-sdk -y
COPY ./secrets/service_account.json /service_account.json
RUN gcloud auth activate-service-account icfpc2022@icfpc-primary.iam.gserviceaccount.com \
        --key-file=/service_account.json \
    && gcloud config set project icfpc-primary

RUN rustup target add x86_64-unknown-linux-musl
RUN mkdir -p /work/src
WORKDIR /work
COPY Cargo.lock /work/Cargo.lock
COPY Cargo.toml /work/Cargo.toml
RUN touch ./src/lib.rs && cargo vendor && cargo build --release && rm -rf ./src
COPY src/ /work/src/
RUN find /work/src -print -exec touch "{}" \; \
    && cargo build --release --bins
COPY scripts/deploy_binaries.sh /work/scripts/deploy_binaries.sh
