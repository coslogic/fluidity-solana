
FROM rust:latest

RUN sh -c "$(curl -sSfL https://release.solana.com/v1.9.5/install)"

ENV PATH="/root/.local/share/solana/install/active_release/bin:$PATH"

WORKDIR /usr/local/src/fluidity-solana

COPY . ./

RUN [ "ls" ]

RUN ["cargo", "build-bpf"]
