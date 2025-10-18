use std assert
use std/testing *

@test
def "Test backward pipeline form - basic" [] {
  let input_data = $in
  let p = (torch full [5] 1 --requires_grad true)
  let loss = ($p | torch sin | torch mean)
  $p | torch zero_grad
  $loss | torch backward
  # After backward, gradient should exist and SGD step should change values
  let before = ($p | torch value | get 0)
  [$p] | torch sgd_step --lr 0.2
  let after = ($p | torch value | get 0)
  assert ($after != $before)
}

@test
def "Test backward argument form" [] {
  let input_data = $in
  let w = (torch full [3] 2 --requires_grad true)
  let loss = ($w | torch mean)
  $w | torch zero_grad
  torch backward $loss
  # Check gradient was populated
  let grad = ($w | torch grad)
  assert ($grad != null)
}

@test
def "Test backward gradient accumulation" [] {
  let input_data = $in
  let x = (torch full [1] 3 --requires_grad true)
  let y = ($x | torch mul $x)  # y = x^2
  $x | torch zero_grad
  $y | torch backward
  # Gradient of x^2 at x=3 should be 2*3 = 6
  let grad_val = ($x | torch grad | torch value | get 0)
  assert ($grad_val == 6)
}

@test
def "Test backward returns same tensor ID" [] {
  let input_data = $in
  let w = (torch full [1] 1 --requires_grad true)
  let loss = ($w | torch mean)
  let result_id = ($loss | torch backward)
  # Should return the same loss tensor ID for chaining
  assert ($result_id == $loss)
}

@test
def "Test backward with scalar loss from reduction" [] {
  let input_data = $in
  let params = (torch full [2 3] 1 --requires_grad true)
  let loss = ($params | torch sin | torch mean)  # Scalar
  $params | torch zero_grad
  $loss | torch backward
  # Gradients should be populated
  let grad = ($params | torch grad)
  assert ($grad != null)
}

@test
def "Error case - non-scalar tensor" [] {
  let input_data = $in
  let t = (torch full [2 2] 1 --requires_grad true)
  try {
    $t | torch backward
    error make {msg: "Expected error from non-scalar tensor"}
  } catch {
    # expected - backward requires scalar loss
  }
}

@test
def "Error case - invalid tensor ID" [] {
  let input_data = $in
  try {
    "invalid-uuid" | torch backward
    error make {msg: "Expected error from invalid tensor ID"}
  } catch {
    # expected
  }
}

@test
def "Error case - vector tensor" [] {
  let input_data = $in
  let v = ([1 2 3] | torch tensor)
  try {
    $v | torch backward
    error make {msg: "Expected error from vector tensor"}
  } catch {
    # expected - must be scalar (numel == 1)
  }
}
