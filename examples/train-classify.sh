#!/bin/zsh
# Issue 0009 acceptance: train linear(2,8)->relu->linear(8,2) with
# cross_entropy on a linearly separable toy set, from plain shell.
# Private socket + seeded init. Usage: scripts/train-classify.sh [torch]
set -e
T=${1:-torch}
S=$(mktemp -d)/nutorchd.sock
trap '$T daemon stop --socket $S >/dev/null 2>&1 || true' EXIT

$T manual_seed 7 --socket $S
x=$($T tensor '[[0.0,0.0],[0.2,0.1],[1.0,1.0],[0.9,0.8],[0.1,0.3],[0.8,1.1]]' --socket $S)
labels=$($T tensor '[0,0,1,1,0,1]' --dtype int64 --socket $S)
l1=$($T nn linear 2 8 --socket $S)
l2=$($T nn linear 8 2 --socket $S)
model=$($T nn sequential $l1 "$($T nn relu --socket $S)" $l2 --socket $S)
opt=$($T nn adam $model --lr 0.05 --socket $S)

first_loss=""
for i in $(seq 100); do
  logits=$($T forward $model $x --socket $S)
  loss=$($T cross_entropy $logits $labels --socket $S)
  if [ -z "$first_loss" ]; then first_loss=$($T value $loss --socket $S); fi
  $T backward $loss --socket $S
  $T step $opt --socket $S
  $T nn zero_grad $opt --socket $S
done
final_loss=$($T value $loss --socket $S)
predictions=$($T argmax $logits --dim 1 --socket $S | $T value --socket $S)
echo "first loss: $first_loss"
echo "final loss: $final_loss"
echo "predictions: $predictions (want [0,0,1,1,0,1])"

python3 - "$first_loss" "$final_loss" "$predictions" <<'PY'
import json, sys
first, final = float(sys.argv[1]), float(sys.argv[2])
preds = json.loads(sys.argv[3])
assert final < first, f"loss did not decrease: {first} -> {final}"
assert preds == [0, 0, 1, 1, 0, 1], f"predictions {preds} != labels"
print("PASS: classification 100% on the toy set")
PY
