use std assert
use std/testing *

@test
def "Test shape with pipeline input" [] {
  let input_data = $in
  let t = torch tensor [1 2 3 4 5]
  let result = ($t | torch shape)
  assert ($result == [5])
}

@test
def "Test shape with argument input" [] {
  let input_data = $in
  let t = torch tensor [1 2 3 4 5]
  let result = (torch shape $t)
  assert ($result == [5])
}

@test
def "Test shape with 2D tensor" [] {
  let input_data = $in
  let t = torch tensor [[1 2 3] [4 5 6]]
  let result = ($t | torch shape)
  assert ($result == [2 3])
}

@test
def "Test shape with 3D tensor" [] {
  let input_data = $in
  let t = torch tensor [[[1 2] [3 4]] [[5 6] [7 8]]]
  let result = (torch shape $t)
  assert ($result == [2 2 2])
}

@test
def "Test shape with scalar" [] {
  let input_data = $in
  let t = torch tensor 42
  let result = ($t | torch shape)
  # Scalar tensor has empty shape
  assert ($result == [])
}

@test
def "Test shape with column vector" [] {
  let input_data = $in
  let t = torch tensor [[1] [2] [3]]
  let result = (torch shape $t)
  assert ($result == [3 1])
}

@test
def "Test shape with row vector" [] {
  let input_data = $in
  let t = torch tensor [[1 2 3]]
  let result = ($t | torch shape)
  assert ($result == [1 3])
}

@test
def "Test shape with rectangular matrix" [] {
  let input_data = $in
  let t = torch tensor [[1 2 3 4] [5 6 7 8] [9 10 11 12]]
  let result = (torch shape $t)
  assert ($result == [3 4])
}

@test
def "Test shape after operations" [] {
  let input_data = $in
  let t1 = torch tensor [[1 2] [3 4]]
  let t2 = torch tensor [[5 6] [7 8]]
  let result = ($t1 | torch add $t2 | torch shape)
  # Shape should be preserved after addition
  assert ($result == [2 2])
}

@test
def "Test shape after transpose" [] {
  let input_data = $in
  let t = torch tensor [[1 2 3] [4 5 6]]
  let result = ($t | torch t | torch shape)
  # 2x3 transposed becomes 3x2
  assert ($result == [3 2])
}

@test
def "Test shape with different sizes" [] {
  let input_data = $in
  let t = torch tensor [[[[1 2 3] [4 5 6]] [[7 8 9] [10 11 12]]]]
  let result = (torch shape $t)
  assert ($result == [1 2 2 3])
}

@test
def "Test shape with single element tensor" [] {
  let input_data = $in
  let t = torch tensor [[[[1]]]]
  let result = ($t | torch shape)
  assert ($result == [1 1 1 1])
}

@test
def "Error case with invalid tensor ID" [] {
  let input_data = $in
  try {
    torch shape "invalid-uuid"
    error make {msg: "Expected error from invalid tensor ID"}
  } catch {
    # expected
  }
}

@test
def "Error case with no tensor provided" [] {
  let input_data = $in
  try {
    torch shape
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
    $t1 | torch shape $t2
    error make {msg: "Expected error from conflicting input"}
  } catch {
    # expected
  }
}
