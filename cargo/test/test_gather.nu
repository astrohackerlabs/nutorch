use std assert
use std/testing *

@test
def "Test gather dim 1 - columns" [] {
  let input_data = $in
  let src = ([[10 11 12] [20 21 22]] | torch tensor)
  let idx = ([[2 1 0] [0 0 2]] | torch tensor --dtype int64)
  let result = ($src | torch gather 1 $idx | torch value)
  # Gather columns: [2,1,0] from row 0, [0,0,2] from row 1
  assert ($result == [[12 11 10] [20 20 22]])
}

@test
def "Test gather dim 0 - rows" [] {
  let input_data = $in
  let src = ([[1 2 3] [4 5 6] [7 8 9]] | torch tensor)
  let idx = ([[0 2 1] [2 0 1]] | torch tensor --dtype int64)
  let result = ($src | torch gather 0 $idx | torch value)
  # Gather along dim 0: [0,2,1] -> [src[0,0], src[2,1], src[1,2]] = [1, 8, 6]
  #                     [2,0,1] -> [src[2,0], src[0,1], src[1,2]] = [7, 2, 6]
  assert ($result == [[1 8 6] [7 2 6]])
}

@test
def "Test gather 1D tensor" [] {
  let input_data = $in
  let src = ([10 20 30 40 50] | torch tensor)
  let idx = ([4 2 0 1] | torch tensor --dtype int64)
  let result = ($src | torch gather 0 $idx | torch value)
  # Gather from 1D: indices [4, 2, 0, 1] -> [50, 30, 10, 20]
  assert ($result == [50 30 10 20])
}

@test
def "Test gather 3D tensor" [] {
  let input_data = $in
  let src = torch full [2 3 4] 1
  let idx = torch full [2 3 2] 1 --dtype int64
  let result = ($src | torch gather 2 $idx | torch shape)
  # Gather along dim 2: [2, 3, 4] with index [2, 3, 2] -> [2, 3, 2]
  assert ($result == [2 3 2])
}

@test
def "Test gather with repeated indices" [] {
  let input_data = $in
  let src = ([[1 2 3] [4 5 6]] | torch tensor)
  let idx = ([[0 0 0] [1 1 1]] | torch tensor --dtype int64)
  let result = ($src | torch gather 1 $idx | torch value)
  # Repeat same index: [0,0,0] and [1,1,1]
  assert ($result == [[1 1 1] [5 5 5]])
}

@test
def "Test gather identity - sequential indices" [] {
  let input_data = $in
  let src = ([[1 2] [3 4]] | torch tensor)
  let idx = ([[0 1] [0 1]] | torch tensor --dtype int64)
  let result = ($src | torch gather 1 $idx | torch value)
  # Sequential indices preserve values
  assert ($result == [[1 2] [3 4]])
}

@test
def "Test gather different output shape" [] {
  let input_data = $in
  let src = ([[1 2 3 4] [5 6 7 8]] | torch tensor)
  let idx = ([[0 2] [1 3]] | torch tensor --dtype int64)
  let result = ($src | torch gather 1 $idx | torch shape)
  # Index shape determines output: [2, 2]
  assert ($result == [2 2])
}

@test
def "Error case - invalid source tensor ID" [] {
  let input_data = $in
  let idx = ([[0 1]] | torch tensor --dtype int64)
  try {
    "invalid-uuid" | torch gather 0 $idx
    error make {msg: "Expected error from invalid source tensor ID"}
  } catch {
    # expected
  }
}

@test
def "Error case - invalid index tensor ID" [] {
  let input_data = $in
  let src = ([[1 2] [3 4]] | torch tensor)
  try {
    $src | torch gather 0 "invalid-uuid"
    error make {msg: "Expected error from invalid index tensor ID"}
  } catch {
    # expected
  }
}

@test
def "Error case - invalid dimension" [] {
  let input_data = $in
  let src = ([[1 2] [3 4]] | torch tensor)
  let idx = ([[0 1]] | torch tensor --dtype int64)
  try {
    $src | torch gather 5 $idx
    error make {msg: "Expected error from invalid dimension"}
  } catch {
    # expected - dim 5 out of bounds for 2D tensor
  }
}

@test
def "Error case - negative dimension" [] {
  let input_data = $in
  let src = ([[1 2] [3 4]] | torch tensor)
  let idx = ([[0 1]] | torch tensor --dtype int64)
  try {
    $src | torch gather (-1) $idx
    error make {msg: "Expected error from negative dimension"}
  } catch {
    # expected
  }
}

@test
def "Error case - rank mismatch" [] {
  let input_data = $in
  let src = ([[1 2] [3 4]] | torch tensor)
  let idx = ([0 0 1] | torch tensor --dtype int64)
  try {
    $src | torch gather 0 $idx
    error make {msg: "Expected error from rank mismatch"}
  } catch {
    # expected - 2D source vs 1D index
  }
}

@test
def "Error case - shape mismatch in non-gather dim" [] {
  let input_data = $in
  let src = ([[1 2 3] [4 5 6]] | torch tensor)
  let idx = ([[0 1] [0 1] [0 1]] | torch tensor --dtype int64)
  try {
    $src | torch gather 1 $idx
    error make {msg: "Expected error from shape mismatch"}
  } catch {
    # expected - dim 0 size mismatch (2 vs 3)
  }
}

@test
def "Error case - index out of range" [] {
  let input_data = $in
  let src = ([[1 2 3] [4 5 6]] | torch tensor)
  let idx = ([[0 1 5]] | torch tensor --dtype int64)
  try {
    $src | torch gather 1 $idx
    error make {msg: "Expected error from index out of range"}
  } catch {
    # expected - index 5 out of range for size 3
  }
}

@test
def "Error case - negative index" [] {
  let input_data = $in
  let src = ([[1 2 3] [4 5 6]] | torch tensor)
  let idx = ([[0 -1 2]] | torch tensor --dtype int64)
  try {
    $src | torch gather 1 $idx
    error make {msg: "Expected error from negative index"}
  } catch {
    # expected - negative indices not allowed
  }
}
