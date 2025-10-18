use std assert
use std/testing *

@test
def "Test mean with pipeline input" [] {
  let input_data = $in
  let t = torch tensor [1 2 3 4 5]
  let result = ($t | torch mean | torch value)
  # Mean of [1,2,3,4,5] = 3
  assert ($result == 3)
}

@test
def "Test mean with argument input" [] {
  let input_data = $in
  let t = torch tensor [1 2 3 4 5]
  let result = (torch mean $t | torch value)
  # Mean of [1,2,3,4,5] = 3
  assert ($result == 3)
}

@test
def "Test mean with 2D tensor all elements" [] {
  let input_data = $in
  let t = torch tensor [[1 2 3] [4 5 6]]
  let result = ($t | torch mean | torch value)
  # Mean of all elements = (1+2+3+4+5+6)/6 = 3.5
  assert ($result == 3.5)
}

@test
def "Test mean with 2D tensor along dim 0" [] {
  let input_data = $in
  let t = torch tensor [[1 2 3] [4 5 6]]
  let result = ($t | torch mean --dim 0 | torch value)
  # Mean along rows: [(1+4)/2, (2+5)/2, (3+6)/2] = [2.5, 3.5, 4.5]
  assert ($result == [2.5 3.5 4.5])
}

@test
def "Test mean with 2D tensor along dim 1" [] {
  let input_data = $in
  let t = torch tensor [[1 2 3] [4 5 6]]
  let result = ($t | torch mean --dim 1 | torch value)
  # Mean along columns: [(1+2+3)/3, (4+5+6)/3] = [2, 5]
  assert ($result == [2 5])
}

@test
def "Test mean with keepdim true" [] {
  let input_data = $in
  let t = torch tensor [[1 2 3] [4 5 6]]
  let result = ($t | torch mean --dim 1 --keepdim true | torch value)
  # Mean with keepdim should preserve dimensions as [[2], [5]]
  assert ($result == [[2] [5]])
}

@test
def "Test mean with keepdim false" [] {
  let input_data = $in
  let t = torch tensor [[1 2 3] [4 5 6]]
  let result = ($t | torch mean --dim 1 --keepdim false | torch value)
  # Mean with keepdim false reduces dimensions to [2, 5]
  assert ($result == [2 5])
}

@test
def "Test mean with floats" [] {
  let input_data = $in
  let t = torch tensor [1.5 2.5 3.5 4.5]
  let result = (torch mean $t | torch value)
  # Mean = (1.5+2.5+3.5+4.5)/4 = 3.0
  assert ($result == 3.0)
}

@test
def "Test mean with negative numbers" [] {
  let input_data = $in
  let t = torch tensor [-2 -1 0 1 2]
  let result = ($t | torch mean | torch value)
  # Mean of [-2,-1,0,1,2] = 0
  assert ($result == 0)
}

@test
def "Test mean with all zeros" [] {
  let input_data = $in
  let t = torch tensor [0 0 0 0]
  let result = (torch mean $t | torch value)
  assert ($result == 0)
}

@test
def "Test mean with 3D tensor" [] {
  let input_data = $in
  let t = torch tensor [[[1 2] [3 4]] [[5 6] [7 8]]]
  let result = ($t | torch mean | torch value)
  # Mean of all elements = (1+2+3+4+5+6+7+8)/8 = 4.5
  assert ($result == 4.5)
}

@test
def "Test mean with 3D tensor along dim 0" [] {
  let input_data = $in
  let t = torch tensor [[[1 2] [3 4]] [[5 6] [7 8]]]
  let result = ($t | torch mean --dim 0 | torch value)
  # Mean along first dimension
  assert ($result == [[3 4] [5 6]])
}

@test
def "Error case with invalid tensor ID" [] {
  let input_data = $in
  try {
    torch mean "invalid-uuid"
    error make {msg: "Expected error from invalid tensor ID"}
  } catch {
    # expected
  }
}

@test
def "Error case with no tensor provided" [] {
  let input_data = $in
  try {
    torch mean
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
    $t1 | torch mean $t2
    error make {msg: "Expected error from conflicting input"}
  } catch {
    # expected
  }
}

@test
def "Error case with invalid dimension" [] {
  let input_data = $in
  try {
    let t = torch tensor [1 2 3]
    $t | torch mean --dim 5
    error make {msg: "Expected error from invalid dimension"}
  } catch {
    # expected
  }
}

@test
def "Error case with negative dimension" [] {
  let input_data = $in
  try {
    let t = torch tensor [1 2 3]
    $t | torch mean --dim -1
    error make {msg: "Expected error from negative dimension"}
  } catch {
    # expected
  }
}
