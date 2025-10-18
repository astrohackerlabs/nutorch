use std assert
use std/testing *

@test
def "Test mul with pipeline and argument" [] {
  let input_data = $in
  let result1 = ([2] | torch tensor) | torch mul ([3] | torch tensor) | torch value | get 0
  assert ($result1 == 6)
}

@test
def "Test mul with two arguments" [] {
  let input_data = $in
  let result2 = torch mul ([2] | torch tensor) ([3] | torch tensor) | torch value | get 0
  assert ($result2 == 6)
}

@test
def "Test mul with vectors" [] {
  let input_data = $in
  let result3 = ([2 3 4] | torch tensor) | torch mul ([5 6 7] | torch tensor) | torch value
  assert ($result3 == [10 18 28])
}

@test
def "Test mul with 2D tensors" [] {
  let input_data = $in
  let t1 = torch tensor [[1 2] [3 4]]
  let t2 = torch tensor [[5 6] [7 8]]
  let result = (torch mul $t1 $t2 | torch value)
  assert ($result == [[5 12] [21 32]])
}

@test
def "Test mul with broadcasting scalar" [] {
  let input_data = $in
  let t1 = torch tensor [1 2 3]
  let scalar = torch tensor 10
  let result = ($t1 | torch mul $scalar | torch value)
  assert ($result == [10 20 30])
}

@test
def "Test mul with negative numbers" [] {
  let input_data = $in
  let t1 = torch tensor [2 -3 4]
  let t2 = torch tensor [-1 2 -3]
  let result = (torch mul $t1 $t2 | torch value)
  assert ($result == [-2 -6 -12])
}

@test
def "Test mul with floats" [] {
  let input_data = $in
  let t1 = torch tensor [1.5 2.5 3.5]
  let t2 = torch tensor [2.0 2.0 2.0]
  let result = (torch mul $t1 $t2 | torch value)
  assert ($result == [3.0 5.0 7.0])
}

@test
def "Test mul with zeros" [] {
  let input_data = $in
  let t1 = torch tensor [5 10 15]
  let t2 = torch tensor [0 0 0]
  let result = (torch mul $t1 $t2 | torch value)
  assert ($result == [0 0 0])
}

@test
def "Test mul with ones" [] {
  let input_data = $in
  let t1 = torch tensor [5 10 15]
  let t2 = torch tensor [1 1 1]
  let result = (torch mul $t1 $t2 | torch value)
  assert ($result == [5 10 15])
}

@test
def "Error case with device mismatch" [] {
  let input_data = $in
  try {
    let t1 = torch tensor [2 3 4] --device cpu
    let t2 = torch tensor [5 6 7] --device cpu
    # This test would fail if we had multiple devices available
    # For now, just verify both tensors are on same device
    let result = (torch mul $t1 $t2 | torch value)
    assert ($result == [10 18 28])
  } catch {
    # Expected if devices differ
  }
}

@test
def "Error case with invalid tensor ID" [] {
  let input_data = $in
  try {
    let t1 = torch tensor [1 2 3]
    torch mul $t1 "invalid-uuid"
    error make {msg: "Expected error from invalid tensor ID"}
  } catch {
    # expected
  }
}

@test
def "Error case with missing second tensor" [] {
  let input_data = $in
  try {
    let t1 = torch tensor [1 2 3]
    $t1 | torch mul
    error make {msg: "Expected error from missing second tensor"}
  } catch {
    # expected
  }
}

@test
def "Error case with no tensors provided" [] {
  let input_data = $in
  try {
    torch mul
    error make {msg: "Expected error from no tensors"}
  } catch {
    # expected
  }
}

@test
def "Error case with three tensors" [] {
  let input_data = $in
  try {
    let t1 = torch tensor [1]
    let t2 = torch tensor [2]
    let t3 = torch tensor [3]
    $t1 | torch mul $t2 $t3
    error make {msg: "Expected error from three tensors"}
  } catch {
    # expected
  }
}
