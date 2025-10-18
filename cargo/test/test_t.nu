use std assert
use std/testing *

@test
def "Test transpose with pipeline input" [] {
  let input_data = $in
  let t = torch tensor [[1 2 3] [4 5 6]]
  let result = ($t | torch t | torch value)
  # [[1 2 3], [4 5 6]] transposed = [[1 4], [2 5], [3 6]]
  assert ($result == [[1 4] [2 5] [3 6]])
}

@test
def "Test transpose with argument input" [] {
  let input_data = $in
  let t = torch tensor [[1 2 3] [4 5 6]]
  let result = (torch t $t | torch value)
  # [[1 2 3], [4 5 6]] transposed = [[1 4], [2 5], [3 6]]
  assert ($result == [[1 4] [2 5] [3 6]])
}

@test
def "Test transpose square matrix" [] {
  let input_data = $in
  let t = torch tensor [[1 2] [3 4]]
  let result = ($t | torch t | torch value)
  assert ($result == [[1 3] [2 4]])
}

@test
def "Test transpose rectangular matrix" [] {
  let input_data = $in
  let t = torch tensor [[1 2 3 4] [5 6 7 8]]
  let result = (torch t $t | torch value)
  # 2x4 becomes 4x2
  assert ($result == [[1 5] [2 6] [3 7] [4 8]])
}

@test
def "Test transpose column vector" [] {
  let input_data = $in
  let t = torch tensor [[1] [2] [3]]
  let result = ($t | torch t | torch value)
  # Column vector becomes row vector
  assert ($result == [[1 2 3]])
}

@test
def "Test transpose row vector" [] {
  let input_data = $in
  let t = torch tensor [[1 2 3]]
  let result = (torch t $t | torch value)
  # Row vector becomes column vector
  assert ($result == [[1] [2] [3]])
}

@test
def "Test transpose with negative numbers" [] {
  let input_data = $in
  let t = torch tensor [[-1 -2] [-3 -4]]
  let result = ($t | torch t | torch value)
  assert ($result == [[-1 -3] [-2 -4]])
}

@test
def "Test transpose with floats" [] {
  let input_data = $in
  let t = torch tensor [[1.5 2.5] [3.5 4.5]]
  let result = (torch t $t | torch value)
  assert ($result == [[1.5 3.5] [2.5 4.5]])
}

@test
def "Test transpose with zeros" [] {
  let input_data = $in
  let t = torch tensor [[0 0] [0 0]]
  let result = ($t | torch t | torch value)
  assert ($result == [[0 0] [0 0]])
}

@test
def "Test transpose double transpose identity" [] {
  let input_data = $in
  let t = torch tensor [[1 2 3] [4 5 6]]
  let result = ($t | torch t | torch t | torch value)
  # Transpose twice returns to original
  assert ($result == [[1 2 3] [4 5 6]])
}

@test
def "Test transpose identity matrix" [] {
  let input_data = $in
  let identity = torch tensor [[1 0 0] [0 1 0] [0 0 1]]
  let result = ($identity | torch t | torch value)
  # Identity matrix transposed is itself
  assert ($result == [[1 0 0] [0 1 0] [0 0 1]])
}

@test
def "Test transpose 1x1 matrix" [] {
  let input_data = $in
  let t = torch tensor [[5]]
  let result = (torch t $t | torch value)
  # 1x1 matrix transposed is itself
  assert ($result == [[5]])
}

@test
def "Error case with invalid tensor ID" [] {
  let input_data = $in
  try {
    torch t "invalid-uuid"
    error make {msg: "Expected error from invalid tensor ID"}
  } catch {
    # expected
  }
}

@test
def "Error case with no tensor provided" [] {
  let input_data = $in
  try {
    torch t
    error make {msg: "Expected error from no tensor"}
  } catch {
    # expected
  }
}

@test
def "Error case with both pipeline and argument" [] {
  let input_data = $in
  try {
    let t1 = torch tensor [[1 2] [3 4]]
    let t2 = torch tensor [[5 6] [7 8]]
    $t1 | torch t $t2
    error make {msg: "Expected error from conflicting input"}
  } catch {
    # expected
  }
}

@test
def "Error case with 1D tensor" [] {
  let input_data = $in
  try {
    let t = torch tensor [1 2 3]
    torch t $t
    error make {msg: "Expected error from 1D tensor"}
  } catch {
    # expected
  }
}

@test
def "Error case with 3D tensor" [] {
  let input_data = $in
  try {
    let t = torch tensor [[[1 2] [3 4]] [[5 6] [7 8]]]
    $t | torch t
    error make {msg: "Expected error from 3D tensor"}
  } catch {
    # expected
  }
}
