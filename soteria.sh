#!/bin/bash

docker run -v $PWD/:/workspace -it greencorelab/soteria:latest /bin/bash -c "export PATH=/root/.cargo/bin:/root/.local/share/solana/install/active_release/bin:/soteria-linux-develop/bin:$PATH; cargo build-bpf; soteria -analyzeAll ."