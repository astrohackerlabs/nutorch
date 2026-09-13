#!/usr/bin/env nutorch
use torch
# Native counterpart of the baseline classification acceptance script.
# Fit linear(2,8) -> relu -> linear(8,2) with Adam and cross-entropy.
torch manual_seed 7
let x = torch tensor [[0.0 0.0] [0.2 0.1] [1.0 1.0] [0.9 0.8] [0.1 0.3] [0.8 1.1]]
let labels = torch tensor [0 0 1 1 0 1] --dtype int64
let first_layer = torch nn linear 2 8
let second_layer = torch nn linear 8 2
let model = torch nn sequential $first_layer (torch nn relu) $second_layer
let optimizer = torch nn adam $model --lr 0.05
mut first_loss = -1.0
for _ in 1..100 {
  let loss = ($x | torch forward $model | torch cross_entropy $labels)
  if $first_loss < 0 { $first_loss = ($loss | torch value) }
  $loss | torch backward
  $optimizer | torch step
  torch nn zero_grad $optimizer
}
let logits = ($x | torch forward $model)
let final_loss = ($logits | torch cross_entropy $labels | torch value)
let predictions = ($logits | torch argmax --dim 1 | torch value)
print $"first loss: ($first_loss)"
print $"final loss: ($final_loss)"
print $"predictions: ($predictions)"
if $final_loss >= $first_loss { error make {msg: 'Classification loss did not decrease'} }
if $predictions != [0 0 1 1 0 1] { error make {msg: $"Predictions ($predictions) do not match labels"} }
print 'PASS: classification 100% on the toy set'
