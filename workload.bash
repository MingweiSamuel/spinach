#!/bin/bash
PORT="$1"
CLIENTS="$2"

cargo run --bin kvs --release -- server "localhost:$PORT" & # | (head; tail)
SERVER_PID=$!

(
    for ((i=1; i<=$CLIENTS; i++)); do
        cargo run --bin kvs --release -- client "localhost:$PORT" workload.txt & # &> /dev/null
    done
    wait
)

sleep 5
kill SERVER_PID
