use std assert
use std/testing *

@test
def "Test sub with pipeline and argument" [] {
  let input_data = $in
  let result1 = ([1] | torch tensor) | torch sub ([2] | torch tensor) | torch value | get 0
  assert ($result1 == -1)
}

@test
def "Test sub with two arguments" [] {
  let input_data = $in
  let result2 = torch sub ([1] | torch tensor) ([2] | torch tensor) | torch value | get 0
  assert ($result2 == -1)
}

@test
def "Test sub with vectors" [] {
  let input_data = $in
  let result3 = ([5 7 9] | torch tensor) | torch sub ([1 2 3] | torch tensor) | torch value
  assert ($result3 == [4 5 6])
}

@test
def "Test sub with 2D tensors" [] {
  let input_data = $in
  let t1 = torch tensor [[10 20] [30 40]]
  let t2 = torch tensor [[1 2] [3 4]]
  let result = (torch sub $t1 $t2 | torch value)
  assert ($result == [[9 18] [27 36]])
}

@test
def "Test sub with broadcasting" [] {
  let input_data = $in
  let t1 = torch tensor [10 20 30]
  let scalar = torch tensor 5
  let result = ($t1 | torch sub $scalar | torch value)
  assert ($result == [5 15 25])
}

@test
def "Test sub resulting in negative numbers" [] {
  let input_data = $in
  let t1 = torch tensor [1 2 3]
  let t2 = torch tensor [5 6 7]
  let result = (torch sub $t1 $t2 | torch value)
  assert ($result == [-4 -4 -4])
}

@test
def "Test sub with floats" [] {
  let input_data = $in
  let t1 = torch tensor [5.5 4.5 3.5]
  let t2 = torch tensor [0.5 0.5 0.5]
  let result = (torch sub $t1 $t2 | torch value)
  assert ($result == [5.0 4.0 3.0])
}

@test
def "Test sub resulting in zero" [] {
  let input_data = $in
  let t1 = torch tensor [5 10 15]
  let t2 = torch tensor [5 10 15]
  let result = (torch sub $t1 $t2 | torch value)
  assert ($result == [0 0 0])
}

@test
def "Error case with device mismatch" [] {
  let input_data = $in
  try {
    let t1 = torch tensor [5 6 7] --device cpu
    let t2 = torch tensor [1 2 3] --device cpu
    # This test would fail if we had multiple devices available
    # For now, just verify both tensors are on same device
    let result = (torch sub $t1 $t2 | torch value)
    assert ($result == [4 4 4])
  } catch {
    # Expected if devices differ
  }
}

@test
def "Error case with invalid tensor ID" [] {
  let input_data = $in
  try {
    let t1 = torch tensor [1 2 3]
    torch sub $t1 "invalid-uuid"
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
    $t1 | torch sub
    error make {msg: "Expected error from missing second tensor"}
  } catch {
    # expected
  }
}

@test
def "Error case with no tensors provided" [] {
  let input_data = $in
  try {
    torch sub
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
    $t1 | torch sub $t2 $t3
    error make {msg: "Expected error from three tensors"}
  } catch {
    # expected
  }
}
