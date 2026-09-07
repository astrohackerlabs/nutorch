#!/bin/zsh
# Issue 0009 acceptance: fit y = 2x + 1 with a linear(1,1) and SGD from
# plain shell. Private socket + seeded init: the thresholds are a
# reproducible gate. Usage: scripts/train-regression.sh [torch-binary]
set -e
T=${1:-torch}
S=$(mktemp -d)/nutorchd.sock
trap '$T daemon stop --socket $S >/dev/null 2>&1 || true' EXIT

$T manual_seed 42 --socket $S
x=$($T tensor '[[0.0],[1.0],[2.0],[3.0]]' --socket $S)
y=$($T tensor '[[1.0],[3.0],[5.0],[7.0]]' --socket $S)   # y = 2x + 1
model=$($T nn linear 1 1 --socket $S)
opt=$($T nn sgd $model --lr 0.05 --socket $S)

first_loss=""
for i in $(seq 200); do
  pred=$($T forward $model $x --socket $S)
  loss=$($T mse_loss $pred $y --socket $S)
  if [ -z "$first_loss" ]; then first_loss=$($T value $loss --socket $S); fi
  $T backward $loss --socket $S
  $T step $opt --socket $S
  $T nn zero_grad $opt --socket $S
done
final_loss=$($T value $loss --socket $S)

params=($($T nn parameters $model --socket $S))
weight=$($T value ${params[1]} --socket $S)
bias=$($T value ${params[2]} --socket $S)
echo "first loss: $first_loss"
echo "final loss: $final_loss"
echo "learned weight: $weight (target 2), bias: $bias (target 1)"

python3 - "$final_loss" "$weight" "$bias" <<'PY'
import json, sys
loss = float(sys.argv[1])
weight = json.loads(sys.argv[2])[0][0]
bias = json.loads(sys.argv[3])[0]
assert loss < 1e-3, f"final loss {loss} >= 1e-3"
assert abs(weight - 2) / 2 < 0.05, f"weight {weight} not within 5% of 2"
assert abs(bias - 1) / 1 < 0.05, f"bias {bias} not within 5% of 1"
print("PASS: regression fit y = 2x + 1")
PY
