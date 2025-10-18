use std assert
use std/testing *

@test
def "Test mm with pipeline and argument" [] {
  let input_data = $in
  let a = torch tensor [[1 2] [3 4]]
  let b = torch tensor [[5 6] [7 8]]
  let result = ($a | torch mm $b | torch value)
  # [[1*5+2*7, 1*6+2*8], [3*5+4*7, 3*6+4*8]] = [[19, 22], [43, 50]]
  assert ($result == [[19 22] [43 50]])
}

@test
def "Test mm with two arguments" [] {
  let input_data = $in
  let a = torch tensor [[1 2] [3 4]]
  let b = torch tensor [[5 6] [7 8]]
  let result = (torch mm $a $b | torch value)
  # [[1*5+2*7, 1*6+2*8], [3*5+4*7, 3*6+4*8]] = [[19, 22], [43, 50]]
  assert ($result == [[19 22] [43 50]])
}

@test
def "Test mm with identity matrix" [] {
  let input_data = $in
  let a = torch tensor [[1 2] [3 4]]
  let identity = torch tensor [[1 0] [0 1]]
  let result = ($a | torch mm $identity | torch value)
  # A * I = A
  assert ($result == [[1 2] [3 4]])
}

@test
def "Test mm with rectangular matrices" [] {
  let input_data = $in
  let a = torch tensor [[1 2 3] [4 5 6]]  # 2x3
  let b = torch tensor [[7 8] [9 10] [11 12]]  # 3x2
  let result = (torch mm $a $b | torch value)
  # Result should be 2x2
  assert ($result == [[58 64] [139 154]])
}

@test
def "Test mm column vector" [] {
  let input_data = $in
  let a = torch tensor [[1 2] [3 4]]
  let b = torch tensor [[5] [6]]
  let result = ($a | torch mm $b | torch value)
  # [[1*5+2*6], [3*5+4*6]] = [[17], [39]]
  assert ($result == [[17] [39]])
}

@test
def "Test mm row vector" [] {
  let input_data = $in
  let a = torch tensor [[1 2]]
  let b = torch tensor [[3] [4]]
  let result = (torch mm $a $b | torch value)
  # [[1*3+2*4]] = [[11]]
  assert ($result == [[11]])
}

@test
def "Test mm with zeros" [] {
  let input_data = $in
  let a = torch tensor [[1 2] [3 4]]
  let zeros = torch tensor [[0 0] [0 0]]
  let result = ($a | torch mm $zeros | torch value)
  assert ($result == [[0 0] [0 0]])
}

@test
def "Test mm with negative numbers" [] {
  let input_data = $in
  let a = torch tensor [[1 -2] [3 -4]]
  let b = torch tensor [[-5 6] [7 -8]]
  let result = (torch mm $a $b | torch value)
  # [[1*-5+-2*7, 1*6+-2*-8], [3*-5+-4*7, 3*6+-4*-8]]
  # = [[-19, 22], [-43, 50]]
  assert ($result == [[-19 22] [-43 50]])
}

@test
def "Test mm with floats" [] {
  let input_data = $in
  let a = torch tensor [[1.5 2.5] [3.5 4.5]]
  let b = torch tensor [[0.5 1.5] [2.5 3.5]]
  let result = ($a | torch mm $b | torch value)
  # 1.5*0.5 + 2.5*2.5 = 0.75 + 6.25 = 7.0
  assert (($result | get 0 | get 0) == 7.0)
}

@test
def "Test mm order matters" [] {
  let input_data = $in
  let a = torch tensor [[1 2] [3 4]]
  let b = torch tensor [[5 6] [7 8]]
  let result_ab = (torch mm $a $b | torch value)
  let result_ba = (torch mm $b $a | torch value)
  # Matrix multiplication is not commutative: AB ≠ BA
  assert ($result_ab != $result_ba)
}

@test
def "Test mm 3x3 matrices" [] {
  let input_data = $in
  let a = torch tensor [[1 2 3] [4 5 6] [7 8 9]]
  let b = torch tensor [[9 8 7] [6 5 4] [3 2 1]]
  let result = ($a | torch mm $b | torch value)
  assert ($result == [[30 24 18] [84 69 54] [138 114 90]])
}

@test
def "Error case with invalid first tensor ID" [] {
  let input_data = $in
  try {
    let b = torch tensor [[1 2] [3 4]]
    torch mm "invalid-uuid" $b
    error make {msg: "Expected error from invalid tensor ID"}
  } catch {
    # expected
  }
}

@test
def "Error case with invalid second tensor ID" [] {
  let input_data = $in
  try {
    let a = torch tensor [[1 2] [3 4]]
    $a | torch mm "invalid-uuid"
    error make {msg: "Expected error from invalid tensor ID"}
  } catch {
    # expected
  }
}

@test
def "Error case with only one tensor" [] {
  let input_data = $in
  try {
    let a = torch tensor [[1 2] [3 4]]
    torch mm $a
    error make {msg: "Expected error from missing second tensor"}
  } catch {
    # expected
  }
}

@test
def "Error case with no tensors" [] {
  let input_data = $in
  try {
    torch mm
    error make {msg: "Expected error from no tensors"}
  } catch {
    # expected
  }
}

@test
def "Error case with incompatible dimensions" [] {
  let input_data = $in
  try {
    let a = torch tensor [[1 2 3] [4 5 6]]  # 2x3
    let b = torch tensor [[7 8] [9 10]]  # 2x2 (should be 3x?)
    torch mm $a $b
    error make {msg: "Expected error from incompatible dimensions"}
  } catch {
    # expected
  }
}

@test
def "Error case with 1D tensor first" [] {
  let input_data = $in
  try {
    let a = torch tensor [1 2 3]
    let b = torch tensor [[4 5] [6 7] [8 9]]
    torch mm $a $b
    error make {msg: "Expected error from 1D first tensor"}
  } catch {
    # expected
  }
}

@test
def "Error case with 1D tensor second" [] {
  let input_data = $in
  try {
    let a = torch tensor [[1 2 3] [4 5 6]]
    let b = torch tensor [7 8 9]
    $a | torch mm $b
    error make {msg: "Expected error from 1D second tensor"}
  } catch {
    # expected
  }
}

@test
def "Error case with 3D tensor" [] {
  let input_data = $in
  try {
    let a = torch tensor [[[1 2] [3 4]] [[5 6] [7 8]]]
    let b = torch tensor [[1 0] [0 1]]
    torch mm $a $b
    error make {msg: "Expected error from 3D tensor"}
  } catch {
    # expected
  }
}
