#!/usr/bin/env bash

export PERF_CTL_FIFO="perf_ctl.fifo"
export PERF_ACK_FIFO="perf_ack.fifo"

# https://pramodkumbhar.com/2024/04/linux-perf-measuring-specific-code-sections-with-pause-resume-apis/
[[ -p $PERF_CTL_FIFO ]] && unlink $PERF_CTL_FIFO
[[ -p $PERF_ACK_FIFO ]] && unlink $PERF_ACK_FIFO

mkfifo $PERF_CTL_FIFO
mkfifo $PERF_ACK_FIFO

perf record \
    --delay -1 \
    --control fifo:${PERF_CTL_FIFO},${PERF_ACK_FIFO} \
    --call-graph dwarf,16384 \
    -F 9997 \
    --strict-freq \
    $@

perf script --input perf.data | ~/.cargo/bin/inferno-collapse-perf | ~/.cargo/bin/inferno-flamegraph > out.svg
