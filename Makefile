
REPO := fluidity-solana

CARGO_BUILD_BPF := cargo build-bpf
CARGO_FUZZ := cargo +nightly fuzz run
CARGO_FUZZ_INIT := cargo +nightly fuzz init

DOCKER_BUILD := docker build
DOCKER_RUN := docker run
FUZZ_FILES := $(shell cargo fuzz list)

SRC_FILES := $(shell find src Xargo.toml Cargo.*)

OUT_BPF := target/deploy/fluidity.so

.PHONY: build clean test

all: build

build: ${OUT_BPF}

${OUT_BPF}: ${SRC_FILES}
	@${CARGO_BUILD_BPF}

docker: ${SRC_FILES} Dockerfile
	@${DOCKER_BUILD} --target base -t fluidity/${REPO} .
	@touch docker

test-validator: ${SRC_FILES} Dockerfile
	@${DOCKER_BUILD} --target test-validator -t fluidity/${REPO}:validator .
	@touch validator

run-test-validator: test-validator
	@${DOCKER_RUN} -p 8899:8899 -p 8900:8900 fluidity/${REPO}:validator

cargo_fuzz: ${SRC_FILES}
	@${CARGO_FUZZ} ${FUZZ_FILES}
	@touch cargo_fuzz

test: cargo_fuzz

clean:
	@rm -rf target docker cargo_fuzz
