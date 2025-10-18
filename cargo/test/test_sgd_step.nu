use std assert
use std/testing *

@test
def "Test sgd_step pipeline form - single parameter" [] {
  let input_data = $in
  let w = (torch full [1] 5 --requires_grad true)
  [$w] | torch zero_grad
  let loss = ($w | torch sin | torch mean)
  $loss | torch backward
  let before = ($w | torch value | get 0)
  [$w] | torch sgd_step --lr 0.1
  let after = ($w | torch value | get 0)
  # After SGD step, parameter should change
  assert ($after != $before)
}

@test
def "Test sgd_step argument form - single parameter" [] {
  let input_data = $in
  let w = (torch full [1] 5 --requires_grad true)
  [$w] | torch zero_grad
  let loss = ($w | torch sin | torch mean)
  $loss | torch backward
  let before = ($w | torch value | get 0)
  torch sgd_step [$w] --lr 0.1
  let after = ($w | torch value | get 0)
  # After SGD step, parameter should change
  assert ($after != $before)
}

@test
def "Test sgd_step multiple parameters" [] {
  let input_data = $in
  let w1 = (torch full [1] 3 --requires_grad true)
  let w2 = (torch full [1] 4 --requires_grad true)
  [$w1 $w2] | torch zero_grad
  # Create loss that depends on both parameters
  let sum = (torch add $w1 $w2)
  let loss = ($sum | torch mean)
  $loss | torch backward
  # Both should have gradients now
  let w1_before = ($w1 | torch value | get 0)
  let w2_before = ($w2 | torch value | get 0)
  [$w1 $w2] | torch sgd_step --lr 0.1
  let w1_after = ($w1 | torch value | get 0)
  let w2_after = ($w2 | torch value | get 0)
  # Both parameters should change
  assert ($w1_after != $w1_before)
  assert ($w2_after != $w2_before)
}

@test
def "Test sgd_step with custom learning rate" [] {
  let input_data = $in
  let w = (torch full [1] 10 --requires_grad true)
  [$w] | torch zero_grad
  let loss = ($w | torch mean)
  $loss | torch backward
  let before = ($w | torch value | get 0)
  # Use larger learning rate
  [$w] | torch sgd_step --lr 0.5
  let after = ($w | torch value | get 0)
  # With lr=0.5 and grad=1, should decrease by 0.5
  assert ((((($before - $after) | math abs) - 0.5) | math abs) < 0.001)
}

@test
def "Test sgd_step default learning rate" [] {
  let input_data = $in
  let w = (torch full [1] 10 --requires_grad true)
  [$w] | torch zero_grad
  let loss = ($w | torch mean)
  $loss | torch backward
  let before = ($w | torch value | get 0)
  # Use default lr (0.01)
  [$w] | torch sgd_step
  let after = ($w | torch value | get 0)
  # With default lr=0.01 and grad=1, should decrease by 0.01
  assert ((((($before - $after) | math abs) - 0.01) | math abs) < 0.001)
}

@test
def "Test sgd_step returns parameter IDs" [] {
  let input_data = $in
  let w1 = (torch full [1] 1 --requires_grad true)
  let w2 = (torch full [1] 2 --requires_grad true)
  [$w1 $w2] | torch zero_grad
  let loss = (torch add $w1 $w2 | torch mean)
  $loss | torch backward
  # Should return list of same IDs
  let result = ([$w1 $w2] | torch sgd_step --lr 0.01)
  assert ($result == [$w1 $w2])
}

@test
def "Test sgd_step with no gradient" [] {
  let input_data = $in
  let w = (torch full [1] 5 --requires_grad true)
  # No backward called, so no gradient
  let before = ($w | torch value | get 0)
  [$w] | torch sgd_step --lr 0.1
  let after = ($w | torch value | get 0)
  # Without gradient, parameter should not change
  assert ($after == $before)
}

@test
def "Test sgd_step gradient descent direction" [] {
  let input_data = $in
  let w = (torch full [1] 3 --requires_grad true)
  [$w] | torch zero_grad
  # For loss = w^2, gradient at w=3 is 2*3=6
  let loss = ($w | torch mul $w)
  $loss | torch backward
  let before = ($w | torch value | get 0)
  [$w] | torch sgd_step --lr 0.1
  let after = ($w | torch value | get 0)
  # Should move in negative gradient direction: 3 - 0.1*6 = 2.4
  let expected = 2.4
  assert ((($after - $expected) | math abs) < 0.001)
}

@test
def "Error case - empty parameter list" [] {
  let input_data = $in
  try {
    [] | torch sgd_step --lr 0.1
    error make {msg: "Expected error from empty list"}
  } catch {
    # expected - parameter list cannot be empty
  }
}

@test
def "Error case - invalid tensor ID" [] {
  let input_data = $in
  try {
    ["invalid-uuid"] | torch sgd_step --lr 0.1
    error make {msg: "Expected error from invalid tensor ID"}
  } catch {
    # expected
  }
}

@test
def "Error case - both pipeline and argument" [] {
  let input_data = $in
  let w1 = (torch full [1] 1 --requires_grad true)
  let w2 = (torch full [1] 2 --requires_grad true)
  try {
    [$w1] | torch sgd_step [$w2] --lr 0.1
    error make {msg: "Expected error from dual input"}
  } catch {
    # expected - cannot provide both pipeline and argument
  }
}

@test
def "Error case - no input provided" [] {
  let input_data = $in
  try {
    torch sgd_step --lr 0.1
    error make {msg: "Expected error from missing input"}
  } catch {
    # expected - must provide parameter list
  }
}
