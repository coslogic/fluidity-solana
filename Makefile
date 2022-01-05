
FILES := $(wildcard src/*.rs)

LIB := target/deploy/fluidity.so

.PHONY: build deploy


build: ${FILES}
	cargo build-bpf
	
deploy: ${LIB}
	solana deploy ${LIB}
