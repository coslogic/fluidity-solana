#!/bin/sh

soteriaImg="greencorelab/soteria:latest"

cargoPath="/root/.cargo/bin"
solanaPath="/root/.local/share/solana/install/active_release/bin"
soteriaPath="/soteria-linux-develop/bin"
exportCmd="export PATH=$cargoPath:\$PATH"

buildBpfCmd="cargo build-bpf"

soteriaCmd="soteria -analyzeAll ../.."

runCommand="$exportCmd && echo \$PATH && $buildBpfCmd && $soteriaCmd"

docker run -v $PWD/:/workspace -it $soteriaImg /bin/sh -c "$runCommand"
