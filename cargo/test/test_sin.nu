use std assert
use std/testing *

@test
def "Test sin with pipeline input" [] {
  let input_data = $in
  let t = torch tensor [0 1.5708 3.1416]
  let result = ($t | torch sin | torch value)
  # sin(0) ≈ 0, sin(π/2) ≈ 1, sin(π) ≈ 0
  assert (($result | get 0) < 0.01)
  assert (($result | get 1) > 0.99)
  assert (($result | get 2) < 0.01)
}

@test
def "Test sin with argument input" [] {
  let input_data = $in
  let t = torch tensor [0 1.5708 3.1416]
  let result = (torch sin $t | torch value)
  # sin(0) ≈ 0, sin(π/2) ≈ 1, sin(π) ≈ 0
  assert (($result | get 0) < 0.01)
  assert (($result | get 1) > 0.99)
  assert (($result | get 2) < 0.01)
}

@test
def "Test sin with 2D tensor" [] {
  let input_data = $in
  let t = torch tensor [[0 1.5708] [3.1416 4.7124]]
  let result = ($t | torch sin | torch value)
  # Verify shape is preserved
  assert (($result | length) == 2)
  assert (($result | get 0 | length) == 2)
}

@test
def "Test sin with negative values" [] {
  let input_data = $in
  let t = torch tensor [-1.5708 0 1.5708]
  let result = ($t | torch sin | torch value)
  # sin(-π/2) ≈ -1, sin(0) = 0, sin(π/2) ≈ 1
  assert (($result | get 0) < -0.99)
  assert (($result | get 1) < 0.01)
  assert (($result | get 2) > 0.99)
}

@test
def "Test sin with zero" [] {
  let input_data = $in
  let t = torch tensor [0 0 0]
  let result = (torch sin $t | torch value)
  assert (($result | get 0) == 0)
  assert (($result | get 1) == 0)
  assert (($result | get 2) == 0)
}

@test
def "Test sin with small values" [] {
  let input_data = $in
  let t = torch tensor [0.1 0.2 0.3]
  let result = ($t | torch sin | torch value)
  # For small x, sin(x) ≈ x, so result should be close to input
  assert (($result | get 0) > 0.09 and ($result | get 0) < 0.11)
  assert (($result | get 1) > 0.19 and ($result | get 1) < 0.21)
  assert (($result | get 2) > 0.29 and ($result | get 2) < 0.31)
}

@test
def "Test sin with large values" [] {
  let input_data = $in
  let t = torch tensor [6.2832 9.4248 12.5664]
  let result = ($t | torch sin | torch value)
  # sin(2π) ≈ 0, sin(3π) ≈ 0, sin(4π) ≈ 0
  assert (($result | get 0) < 0.01 and ($result | get 0) > -0.01)
  assert (($result | get 1) < 0.01 and ($result | get 1) > -0.01)
  assert (($result | get 2) < 0.01 and ($result | get 2) > -0.01)
}

@test
def "Test sin chaining" [] {
  let input_data = $in
  let t = torch tensor [0 0.5 1.0]
  let result = ($t | torch sin | torch sin | torch value)
  # sin(sin(x)) should be valid
  assert (($result | length) == 3)
}

@test
def "Error case with invalid tensor ID" [] {
  let input_data = $in
  try {
    torch sin "invalid-uuid"
    error make {msg: "Expected error from invalid tensor ID"}
  } catch {
    # expected
  }
}

@test
def "Error case with no tensor provided" [] {
  let input_data = $in
  try {
    torch sin
    error make {msg: "Expected error from no tensor"}
  } catch {
    # expected
  }
}

@test
def "Error case with both pipeline and argument" [] {
  let input_data = $in
  try {
    let t1 = torch tensor [1 2 3]
    let t2 = torch tensor [4 5 6]
    $t1 | torch sin $t2
    error make {msg: "Expected error from conflicting input"}
  } catch {
    # expected
  }
}
