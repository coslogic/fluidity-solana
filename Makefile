iREPO := fluidity-solana

CARGO_BUILD_BPF := cargo build-bpf

CARGO_TEST := cargo test

TESTING_DIR := testing

CARGO_FUZZ := ${TESTING_DIR}/fuzz/run.sh

SOTERIA_ANALYZE := ${TESTING_DIR}/soteria/soteria.sh

MIRI_DOCKERFILE := ${TESTING_DIR}/miri/Dockerfile

CARGO_GEIGER_DOCKERFILE := ${TESTING_DIR}/cargo_geiger/Dockerfile

CARGO_AUDIT_DOCKERFILE := ${TESTING_DIR}/cargo_audit/Dockerfile

DOCKER_BUILD := docker build
DOCKER_RUN := docker run

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

cargo_geiger: ${SRC_FILES}
	@${DOCKER_BUILD} -q -f ${CARGO_GEIGER_DOCKERFILE} -t flu/cargo-geiger .
	@${DOCKER_RUN} --rm flu/cargo-geiger
	@touch cargo_geiger

cargo_audit: ${SRC_FILES}
	@${DOCKER_BUILD} -q -f ${CARGO_AUDIT_DOCKERFILE} -t flu/cargo-audit .
	@${DOCKER_RUN} --rm flu/cargo-audit
	@touch cargo_audit

miri_test: ${SRC_FILES}
	@${DOCKER_BUILD} -q -f ${MIRI_DOCKERFILE} -t flu/miri .
	@${DOCKER_RUN} --rm flu/miri
	@touch miri_test

test: cargo_test cargo_fuzz soteria_analyze miri_test cargo_geiger cargo_audit

clean:
	@rm -rf \
		target \
		docker \
		cargo_fuzz \
		cargo_test \
		soteria_analyze \
		.coderrect \
		miri_test \
		cargo_geiger \
		cargo_audit
