#!/usr/bin/env bash
set -euo pipefail

ticks="${TICKS:-30000}"
start_pop="${START_POP:-8}"
seeds="${SEEDS:-0 1 2 3 4 5 6 7 8 9}"

cargo build --release >/dev/null

first=1
for seed in $seeds; do
  if [ "$first" -eq 1 ]; then
    target/release/thronglets --headless --csv --seed "$seed" --ticks "$ticks" --start-pop "$start_pop"
    first=0
  else
    target/release/thronglets --headless --csv --seed "$seed" --ticks "$ticks" --start-pop "$start_pop" | tail -n +2
  fi
done
