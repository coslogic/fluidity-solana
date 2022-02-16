#!/bin/bash
set -o errexit

if [ $# -eq 0 ]
then
    echo "Running all fuzzers:"
    cargo fuzz list
    fuzz_targets=$(cargo fuzz list)
else
    echo "Running fuzzers: $@"
    fuzz_targets=$@
fi 

fuzz_targets=$(echo $fuzz_targets | tr '\n' ' ' | tr ',' ' ')

cargo +nightly fuzz run $fuzz_targets -- -max_total_time=300

echo "All done - All fuzz targets exited normally"
