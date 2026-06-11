#!/usr/bin/env nu
# Issue 0010: the Nushell twin of train-regression.sh — fit y = 2x + 1
# with linear(1,1) + SGD, natively. Private socket (TMPDIR) + seeded init.
# Usage: PATH must contain `torch`; run: nu scripts/train-regression.nu

use ../nutorch.nu *

$env.TMPDIR = (mktemp -d)

nutorch manual_seed 42 | ignore
let x = ([[0.0] [1.0] [2.0] [3.0]] | nutorch tensor)
let y = ([[1.0] [3.0] [5.0] [7.0]] | nutorch tensor)   # y = 2x + 1
let model = (nutorch nn linear 1 1)
let opt = (nutorch nn sgd $model --lr 0.05)

mut first_loss = -1.0
mut final_loss = -1.0
for i in 1..200 {
  let loss = ($x | nutorch forward $model | nutorch mse_loss $y)
  let v = ($loss | nutorch value)
  if $first_loss < 0 { $first_loss = $v }
  $final_loss = $v
  $loss | nutorch backward
  $opt | nutorch step
  nutorch nn zero_grad $opt | ignore
}

let params = (nutorch nn parameters $model | lines)
let weight = ($params | first | nutorch value | get 0.0)
let bias = ($params | get 1 | nutorch value | first)
print $"first loss: ($first_loss)"
print $"final loss: ($final_loss)"
print $"learned weight: ($weight) target 2, bias: ($bias) target 1"

nutorch daemon stop | ignore

if $final_loss >= 1e-3 { error make { msg: $"final loss ($final_loss) >= 1e-3" } }
if (($weight - 2.0) | math abs) / 2.0 >= 0.05 { error make { msg: $"weight ($weight) not within 5% of 2" } }
if (($bias - 1.0) | math abs) >= 0.05 { error make { msg: $"bias ($bias) not within 5% of 1" } }
print "PASS: regression fit y = 2x + 1 (nushell)"
