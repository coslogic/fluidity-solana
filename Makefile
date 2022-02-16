
REPO := fluidity-solana

CARGO_BUILD_BPF := cargo build-bpf

CARGO_TEST := cargo test

TESTING_DIR := ${PWD}/testing
CARGO_FUZZ := ${TESTING_DIR}/fuzz/run.sh

SOTERIA_ANALYZE := ${TESTING_DIR}/soteria/soteria.sh

DOCKER_BUILD := docker build

SRC_FILES := $(shell find src Xargo.toml Cargo.*)

OUT_BPF := target/deploy/fluidity.so

.PHONY: build clean test

all: build

build: ${OUT_BPF}

${OUT_BPF}: ${SRC_FILES}
	@${CARGO_BUILD_BPF}

docker: ${SRC_FILES} Dockerfile
	@${DOCKER_BUILD} -t ${REPO} .
	@touch docker

cargo_fuzz: ${SRC_FILES}
	@${CARGO_FUZZ}
	@touch cargo_fuzz

cargo_test: ${SRC_FILES}
	@${CARGO_TEST}
	@touch cargo_test
	
soteria_analyze: ${SRC_FILES}
	@${SOTERIA_ANALYZE}
	@touch soteria_analyze

test: cargo_test cargo_fuzz soteria_analyze

clean:
	@rm -rf target docker cargo_fuzz cargo_test soteria_analyze
