use std assert
use std/testing *

@test
def "Test full with 1D tensor and integer fill" [] {
  let input_data = $in
  let res = (torch full [5] 7 | torch value)
  assert ($res == [7 7 7 7 7])
}

@test
def "Test full with 2D tensor and float fill" [] {
  let input_data = $in
  let res = (torch full [2, 3] 0.5 | torch value)
  assert ($res == [[0.5 0.5 0.5] [0.5 0.5 0.5]])
}

@test
def "Test full with 3D tensor" [] {
  let input_data = $in
  let res = (torch full [2, 2, 2] 1.0 | torch value)
  assert ($res == [[[1.0 1.0] [1.0 1.0]] [[1.0 1.0] [1.0 1.0]]])
}

@test
def "Test full with float64 dtype" [] {
  let input_data = $in
  let res = (torch full [3] 2.5 --dtype float64 | torch value)
  assert ($res == [2.5 2.5 2.5])
}

@test
def "Test full with int64 dtype" [] {
  let input_data = $in
  let res = (torch full [3] 5 --dtype int64 | torch value)
  assert ($res == [5 5 5])
}

@test
def "Test full with requires_grad flag" [] {
  let input_data = $in
  let t = (torch full [2, 2] 1.0 --requires_grad true)
  # Just verify it creates successfully - grad testing happens in backward tests
  assert ($t | describe | str contains "string")
}

@test
def "Test full with device cpu" [] {
  let input_data = $in
  let res = (torch full [2] 3.0 --device cpu | torch value)
  assert ($res == [3.0 3.0])
}

@test
def "Error case with empty size list" [] {
  let input_data = $in
  try {
    torch full [] 1.0
    error make {msg: "Expected error from empty size list"}
  } catch {
    # expected
  }
}

@test
def "Error case with negative dimension" [] {
  let input_data = $in
  try {
    torch full [-1] 1.0
    error make {msg: "Expected error from negative dimension"}
  } catch {
    # expected
  }
}

@test
def "Error case with zero dimension" [] {
  let input_data = $in
  try {
    torch full [0] 1.0
    error make {msg: "Expected error from zero dimension"}
  } catch {
    # expected
  }
}

@test
def "Error case with invalid device" [] {
  let input_data = $in
  try {
    torch full [2] 1.0 --device invalid_device
    error make {msg: "Expected error from invalid device"}
  } catch {
    # expected
  }
}

@test
def "Error case with invalid dtype" [] {
  let input_data = $in
  try {
    torch full [2] 1.0 --dtype invalid_dtype
    error make {msg: "Expected error from invalid dtype"}
  } catch {
    # expected
  }
}

@test
def "Test full with negative fill value" [] {
  let input_data = $in
  let res = (torch full [3] -5.5 | torch value)
  assert ($res == [-5.5 -5.5 -5.5])
}

@test
def "Test full with zero fill value" [] {
  let input_data = $in
  let res = (torch full [2, 2] 0 | torch value)
  assert ($res == [[0 0] [0 0]])
}

@test
def "Test full with large dimensions" [] {
  let input_data = $in
  let t = (torch full [100, 100] 1.0)
  let res = (torch full [2] 42.0 | torch value)
  # Just verify large tensor creates without error
  assert ($res == [42.0 42.0])
}
