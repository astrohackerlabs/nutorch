use std assert
use std/testing *

@test
def "Test value scalar-like tensor" [] {
  let input_data = $in
  # Create a single-element tensor (closest to scalar)
  let t = (torch full [1] 42)
  let result = ($t | torch value)
  # Single element tensor returns a list with one item
  assert ($result == [42])
}

@test
def "Test value 1D tensor via pipeline" [] {
  let input_data = $in
  let t = ([1 2 3] | torch tensor)
  let result = ($t | torch value)
  # Should return a list
  assert ($result == [1 2 3])
}

@test
def "Test value 1D tensor via argument" [] {
  let input_data = $in
  let t = ([1 2 3] | torch tensor)
  let result = ($t | torch value)
  # Should return a list
  assert ($result == [1 2 3])
}

@test
def "Test value 2D tensor" [] {
  let input_data = $in
  let t = ([[1 2] [3 4]] | torch tensor)
  let result = ($t | torch value)
  # Should return nested list
  assert ($result == [[1 2] [3 4]])
}

@test
def "Test value 3D tensor" [] {
  let input_data = $in
  let t = ([[[1 2] [3 4]] [[5 6] [7 8]]] | torch tensor)
  let result = ($t | torch value)
  # Should return deeply nested list
  assert ($result == [[[1 2] [3 4]] [[5 6] [7 8]]])
}

@test
def "Test value with float tensor" [] {
  let input_data = $in
  let t = ([1.5 2.5 3.5] | torch tensor)
  let result = ($t | torch value)
  # Should handle floats
  assert ($result.0 == 1.5)
  assert ($result.1 == 2.5)
  assert ($result.2 == 3.5)
}

@test
def "Test value with linspace" [] {
  let input_data = $in
  let t = (torch linspace 0 1 3)
  let result = ($t | torch value)
  # Should convert linspace result
  assert ($result.0 == 0.0)
  assert ($result.2 == 1.0)
}

@test
def "Test value preserves data after operations" [] {
  let input_data = $in
  let t = ([1 2 3] | torch tensor)
  # Multiply by a tensor containing 2
  let two = (torch full [3] 2)
  let doubled = ($t | torch mul $two)
  let result = ($doubled | torch value)
  # Should reflect the multiplication
  assert ($result == [2 4 6])
}

@test
def "Test value with negative numbers" [] {
  let input_data = $in
  let t = ([-1 -2 -3] | torch tensor)
  let result = ($t | torch value)
  assert ($result == [-1 -2 -3])
}

@test
def "Test value with single element tensor" [] {
  let input_data = $in
  let t = ([42] | torch tensor)
  let result = ($t | torch value)
  # Single element in 1D tensor should return a list with one element
  assert ($result == [42])
}

@test
def "Error case - invalid tensor ID" [] {
  let input_data = $in
  try {
    "invalid-uuid" | torch value
    error make {msg: "Expected error from invalid tensor ID"}
  } catch {
    # expected
  }
}

@test
def "Error case - both pipeline and argument" [] {
  let input_data = $in
  let t1 = ([1 2 3] | torch tensor)
  let t2 = ([4 5 6] | torch tensor)
  try {
    $t1 | $t2 | torch value
    error make {msg: "Expected error from dual input"}
  } catch {
    # expected - cannot provide both pipeline and argument
  }
}

@test
def "Error case - no input provided" [] {
  let input_data = $in
  try {
    torch value
    error make {msg: "Expected error from missing input"}
  } catch {
    # expected - must provide tensor ID
  }
}
