use std assert
use std/testing *

@test
def "Test grad pipeline form - gradient exists" [] {
  let input_data = $in
  let w = (torch full [1] 2 --requires_grad true)
  [$w] | torch zero_grad
  let loss = ($w | torch mean)
  $loss | torch backward
  # After backward, gradient should be defined
  let gid = ($w | torch grad)
  assert ($gid != null)
}

@test
def "Test grad argument form - gradient exists" [] {
  let input_data = $in
  let w = (torch full [1] 2 --requires_grad true)
  [$w] | torch zero_grad
  let loss = ($w | torch mean)
  $loss | torch backward
  # After backward, gradient should be defined
  let gid = (torch grad $w)
  assert ($gid != null)
}

@test
def "Test grad pipeline form - null gradient" [] {
  let input_data = $in
  let v = (torch full [1] 7 --requires_grad true)
  # No backward called, so gradient should be null
  let gnull = ($v | torch grad)
  assert ($gnull == null)
}

@test
def "Test grad argument form - null gradient" [] {
  let input_data = $in
  let v = (torch full [1] 7 --requires_grad true)
  # No backward called, so gradient should be null
  let gnull = (torch grad $v)
  assert ($gnull == null)
}

@test
def "Test grad value verification - sin derivative" [] {
  let input_data = $in
  let xval = 0.5
  let x = (torch full [1] $xval --requires_grad true)
  [$x] | torch zero_grad
  let loss = ($x | torch sin | torch mean)
  $loss | torch backward
  # Gradient of sin(x) at x=0.5 should be cos(0.5)
  let g_id = ($x | torch grad)
  let grad_val = ($g_id | torch value | get 0)
  let expected = ($xval | math cos)
  let diff = (if ($grad_val > $expected) { $grad_val - $expected } else { $expected - $grad_val })
  assert ($diff < 1e-6)
}

@test
def "Test grad multi-dimensional tensor" [] {
  let input_data = $in
  let params = (torch full [2 3] 1 --requires_grad true)
  [$params] | torch zero_grad
  let loss = ($params | torch mean)
  $loss | torch backward
  # Gradient should exist and have same shape as params
  let grad_id = ($params | torch grad)
  assert ($grad_id != null)
  let grad_shape = ($grad_id | torch shape)
  assert ($grad_shape == [2 3])
}

@test
def "Test grad after zero_grad" [] {
  let input_data = $in
  let w = (torch full [1] 3 --requires_grad true)
  # First backward pass
  let loss1 = ($w | torch mean)
  $loss1 | torch backward
  let grad1 = ($w | torch grad)
  assert ($grad1 != null)
  # Zero out gradients
  [$w] | torch zero_grad
  # Gradient should now be defined but zero
  let grad2 = ($w | torch grad)
  assert ($grad2 != null)
  let grad_val = ($grad2 | torch value | get 0)
  assert ($grad_val == 0)
}

@test
def "Test grad returns tensor ID for chaining" [] {
  let input_data = $in
  let x = (torch full [1] 5 --requires_grad true)
  [$x] | torch zero_grad
  let loss = ($x | torch mul $x)
  $loss | torch backward
  # Should return tensor ID that can be used with other commands
  let grad_id = ($x | torch grad)
  let grad_shape = ($grad_id | torch shape)
  assert ($grad_shape == [1])
}

@test
def "Error case - invalid tensor ID" [] {
  let input_data = $in
  try {
    "invalid-uuid" | torch grad
    error make {msg: "Expected error from invalid tensor ID"}
  } catch {
    # expected
  }
}

@test
def "Error case - both pipeline and argument" [] {
  let input_data = $in
  let t1 = (torch full [1] 1 --requires_grad true)
  let t2 = (torch full [1] 2 --requires_grad true)
  try {
    $t1 | torch grad $t2
    error make {msg: "Expected error from dual input"}
  } catch {
    # expected - cannot provide both pipeline and argument
  }
}

@test
def "Error case - no input provided" [] {
  let input_data = $in
  try {
    torch grad
    error make {msg: "Expected error from missing input"}
  } catch {
    # expected - must provide tensor ID
  }
}
