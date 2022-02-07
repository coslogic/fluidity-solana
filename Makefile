
REPO := fluidity-solana

CARGO_BUILD_BPF := cargo build-bpf
DOCKER_BUILD := docker build

CARGO_FUZZ := cargo +nightly fuzz run
CARGO_FUZZ_INIT := cargo +nightly fuzz init
FUZZ_FILES := $(shell cargo fuzz list)

SRC_FILES := $(shell find src Xargo.toml Cargo.*)

OUT_BPF := target/deploy/fluidity.so

.PHONY: build clean

all: build

build: ${OUT_BPF}

${OUT_BPF}: ${SRC_FILES}
	@${CARGO_BUILD_BPF}

docker: ${SRC_FILES} Dockerfile
	@${DOCKER_BUILD} -t ${REPO} .
	@touch docker

fuzzy:
	@${CARGO_FUZZ} ${FUZZ_FILES}

clean:
	@rm -rf target docker
