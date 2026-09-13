#!/usr/bin/env nutorch
use torch
# Fit y = 2x + 1 with native linear(1,1) + SGD and seeded initialization.
# From the monorepo: ./code/nutorch/rs/target/release/nutorch code/nutorch/examples/train-regression.nu

torch manual_seed 42 | ignore
let x = ([[0.0] [1.0] [2.0] [3.0]] | torch tensor)
let y = ([[1.0] [3.0] [5.0] [7.0]] | torch tensor)   # y = 2x + 1
let model = (torch nn linear 1 1)
let opt = (torch nn sgd $model --lr 0.05)

mut first_loss = -1.0
mut final_loss = -1.0
for i in 1..200 {
  let loss = ($x | torch forward $model | torch mse_loss $y)
  let v = ($loss | torch value)
  if $first_loss < 0 { $first_loss = $v }
  $final_loss = $v
  $loss | torch backward
  $opt | torch step
  torch nn zero_grad $opt | ignore
}

let params = (torch nn parameters $model)
let weight = ($params | first | torch value | get 0.0)
let bias = ($params | get 1 | torch value | first)
print $"first loss: ($first_loss)"
print $"final loss: ($final_loss)"
print $"learned weight: ($weight) target 2, bias: ($bias) target 1"

if $final_loss >= 1e-3 { error make { msg: $"final loss ($final_loss) >= 1e-3" } }
if (($weight - 2.0) | math abs) / 2.0 >= 0.05 { error make { msg: $"weight ($weight) not within 5% of 2" } }
if (($bias - 1.0) | math abs) >= 0.05 { error make { msg: $"bias ($bias) not within 5% of 1" } }
print "PASS: regression fit y = 2x + 1 (nushell)"
