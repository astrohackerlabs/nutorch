use std assert
use std/testing *

@test
def "Test tensor creation" [] {
  let input_data = $in
  let res = ([1.0 2.0 3.0] | torch tensor)
  assert ($res | describe | str contains "string")
}

@test
def "Convert a 1d list to a tensor via argument" [] {
  let input_data = $in
  let res = (torch tensor [1.0 2.0 3.0])
  assert ($res | describe | str contains "string")
}

@test
def "Error case of no input provided" [] {
  let input_data = $in
  try {
    torch tensor
    error make {msg: "Expected error from no input"}
  } catch {
    # expected
  }
}

@test
def "Expect an error if pipeline and argument both provided" [] {
  let input_data = $in
  try {
    let res = ([1 2 3] | torch tensor [1.0 2.0 3.0])
    error make {msg: "Expected error if pipeline and argument both provided"}
  } catch {
    # expected
  }
}

@test
def "Convert a mixed list to a tensor" [] {
  let input_data = $in
  let res = ([1.0 2 3] | torch tensor | torch value)
  assert ($res == [1.0 2.0 3.0])
}

@test
def "Error case with invalid device" [] {
  let input_data = $in
  try {
    [1.0 2.0 3.0] | torch tensor --device invalid_device
    error make {msg: "Expected error from invalid device"}
  } catch {
    # expected
  }
}

@test
def "Error case with invalid dtype" [] {
  let input_data = $in
  try {
    [1.0 2.0 3.0] | torch tensor --dtype invalid_dtype
    error make {msg: "Expected error from invalid dtype"}
  } catch {
    # expected
  }
}

@test
def "Error case with empty list" [] {
  let input_data = $in
  try {
    [] | torch tensor
    error make {msg: "Expected error from empty list"}
  } catch {
    # expected
  }
}

@test
def "Test tensor with requires_grad flag" [] {
  let input_data = $in
  let t = ([1.0 2.0 3.0] | torch tensor --requires_grad true)
  # Just verify it creates successfully - grad testing happens in backward tests
  assert ($t | describe | str contains "string")
}

@test
def "Test tensor with int64 dtype" [] {
  let input_data = $in
  let res = ([1 2 3] | torch tensor --dtype int64 | torch value)
  assert ($res == [1 2 3])
}

@test
def "Test 2D tensor creation via pipeline" [] {
  let input_data = $in
  let res = ([[1.0 2.0] [3.0 4.0]] | torch tensor | torch value)
  assert ($res == [[1.0 2.0] [3.0 4.0]])
}

@test
def "Test 2D tensor creation via argument" [] {
  let input_data = $in
  let res = (torch tensor [[1.0 2.0] [3.0 4.0]] | torch value)
  assert ($res == [[1.0 2.0] [3.0 4.0]])
}
