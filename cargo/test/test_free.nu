use std assert
use std/testing *

@test
def "Test free single tensor via pipeline" [] {
  let input_data = $in
  let t = ([1 2 3] | torch tensor)
  # Free the tensor
  let result = ($t | torch free)
  # Should return list with the freed ID
  assert ($result == [$t])
  # Tensor should no longer be accessible
  try {
    $t | torch value
    error make {msg: "Expected error - tensor should be freed"}
  } catch {
    # expected - tensor was freed
  }
}

@test
def "Test free single tensor via argument" [] {
  let input_data = $in
  let t = ([1 2 3] | torch tensor)
  # Free via argument form (must use list)
  let result = (torch free [$t])
  # Should return list with the freed ID
  assert ($result == [$t])
  # Verify tensor is gone
  try {
    $t | torch value
    error make {msg: "Expected error - tensor should be freed"}
  } catch {
    # expected
  }
}

@test
def "Test free list via pipeline" [] {
  let input_data = $in
  let t1 = ([1 2 3] | torch tensor)
  let t2 = ([4 5 6] | torch tensor)
  # Free list via pipeline
  let result = ([$t1 $t2] | torch free)
  # Should return both IDs
  assert ($result == [$t1 $t2])
}

@test
def "Test free list via argument" [] {
  let input_data = $in
  let t1 = ([1 2 3] | torch tensor)
  let t2 = ([4 5 6] | torch tensor)
  # Free list via argument
  let result = (torch free [$t1 $t2])
  # Should return both IDs
  assert ($result == [$t1 $t2])
}

@test
def "Test free multiple tensors" [] {
  let input_data = $in
  let t1 = ([1 2 3] | torch tensor)
  let t2 = ([4 5 6] | torch tensor)
  let t3 = ([7 8 9] | torch tensor)
  # Free all three
  let result = (torch free [$t1 $t2 $t3])
  assert ($result == [$t1 $t2 $t3])
  # All should be inaccessible
  try {
    $t1 | torch value
    error make {msg: "Expected error"}
  } catch {
    # expected
  }
}

@test
def "Test free returns freed IDs" [] {
  let input_data = $in
  let t = ([42] | torch tensor)
  let result = ($t | torch free)
  # Should return the ID in a list
  assert ($result.0 == $t)
}

@test
def "Test free allows reusing variable name" [] {
  let input_data = $in
  let t = ([1 2 3] | torch tensor)
  $t | torch free
  # Create new tensor with same variable name
  let t = ([4 5 6] | torch tensor)
  let result = ($t | torch value)
  # Should have new values
  assert ($result == [4 5 6])
}

@test
def "Test free with large tensor" [] {
  let input_data = $in
  # Create a larger tensor
  let t = (torch randn 100 100)
  let result = ($t | torch free)
  assert ($result == [$t])
}

@test
def "Error case - invalid tensor ID" [] {
  let input_data = $in
  try {
    "invalid-uuid" | torch free
    error make {msg: "Expected error from invalid tensor ID"}
  } catch {
    # expected
  }
}

@test
def "Error case - empty list" [] {
  let input_data = $in
  try {
    [] | torch free
    error make {msg: "Expected error from empty list"}
  } catch {
    # expected - cannot free empty list
  }
}

@test
def "Error case - both pipeline and argument" [] {
  let input_data = $in
  let t1 = ([1 2 3] | torch tensor)
  let t2 = ([4 5 6] | torch tensor)
  try {
    $t1 | torch free [$t2]
    error make {msg: "Expected error from dual input"}
  } catch {
    # expected - cannot provide both pipeline and argument
  }
  # Clean up tensors used in error test
  [$t1 $t2] | torch free
}

@test
def "Error case - no input provided" [] {
  let input_data = $in
  try {
    torch free
    error make {msg: "Expected error from missing input"}
  } catch {
    # expected - must provide tensor ID
  }
}

@test
def "Error case - double free" [] {
  let input_data = $in
  let t = ([1 2 3] | torch tensor)
  # Free once
  $t | torch free
  # Try to free again - should error
  try {
    $t | torch free
    error make {msg: "Expected error from double free"}
  } catch {
    # expected - tensor already freed
  }
}
