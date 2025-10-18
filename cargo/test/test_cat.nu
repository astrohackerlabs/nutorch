use std assert
use std/testing *

@test
def "Test cat dim 0 - argument form" [] {
  let input_data = $in
  let t1 = (torch full [2 3] 1)
  let t2 = (torch full [2 3] 2)
  let result = (torch cat [$t1 $t2] --dim 0 | torch value)
  # [2, 3] + [2, 3] along dim 0 = [4, 3]
  assert ($result == [[1 1 1] [1 1 1] [2 2 2] [2 2 2]])
}

@test
def "Test cat dim 1 - argument form" [] {
  let input_data = $in
  let t1 = (torch full [2 3] 1)
  let t2 = (torch full [2 3] 3)
  let result = (torch cat [$t1 $t2] --dim 1 | torch value)
  # [2, 3] + [2, 3] along dim 1 = [2, 6]
  assert ($result == [[1 1 1 3 3 3] [1 1 1 3 3 3]])
}

@test
def "Test cat dim 0 - pipeline form" [] {
  let input_data = $in
  let t1 = (torch full [2 3] 1)
  let t2 = (torch full [2 3] 2)
  let result = ([$t1 $t2] | torch cat --dim 0 | torch value)
  # Pipeline input should work the same as argument
  assert ($result == [[1 1 1] [1 1 1] [2 2 2] [2 2 2]])
}

@test
def "Test cat three tensors" [] {
  let input_data = $in
  let t1 = (torch full [2 2] 1)
  let t2 = (torch full [2 2] 2)
  let t3 = (torch full [2 2] 3)
  let result = (torch cat [$t1 $t2 $t3] --dim 0 | torch shape)
  # [2, 2] + [2, 2] + [2, 2] along dim 0 = [6, 2]
  assert ($result == [6 2])
}

@test
def "Test cat default dim is 0" [] {
  let input_data = $in
  let t1 = (torch full [3 2] 1)
  let t2 = (torch full [3 2] 2)
  let result = (torch cat [$t1 $t2] | torch shape)
  # Default dim should be 0
  assert ($result == [6 2])
}

@test
def "Test cat 3D tensors" [] {
  let input_data = $in
  let t1 = torch full [2 3 4] 1
  let t2 = torch full [2 3 4] 2
  let result = (torch cat [$t1 $t2] --dim 1 | torch shape)
  # [2, 3, 4] + [2, 3, 4] along dim 1 = [2, 6, 4]
  assert ($result == [2 6 4])
}

@test
def "Test cat 1D tensors" [] {
  let input_data = $in
  let t1 = ([1 2 3] | torch tensor)
  let t2 = ([4 5 6] | torch tensor)
  let result = (torch cat [$t1 $t2] | torch value)
  # [3] + [3] = [6]
  assert ($result == [1 2 3 4 5 6])
}

@test
def "Error case - incompatible shapes" [] {
  let input_data = $in
  let t1 = (torch full [2 3] 1)
  let t4 = (torch full [2 2] 4)
  try {
    torch cat [$t1 $t4] --dim 0 | torch value
    error make {msg: "Expected error from incompatible shapes"}
  } catch {
    # expected - dimension 1 doesn't match (3 vs 2)
  }
}

@test
def "Error case - invalid tensor ID" [] {
  let input_data = $in
  let t1 = (torch full [2 3] 1)
  try {
    torch cat [$t1 "invalid-uuid"]
    error make {msg: "Expected error from invalid tensor ID"}
  } catch {
    # expected
  }
}

@test
def "Error case - only one tensor" [] {
  let input_data = $in
  let t1 = (torch full [2 3] 1)
  try {
    torch cat [$t1]
    error make {msg: "Expected error from only one tensor"}
  } catch {
    # expected - need at least 2 tensors
  }
}

@test
def "Error case - invalid dimension" [] {
  let input_data = $in
  let t1 = (torch full [2 3] 1)
  let t2 = (torch full [2 3] 2)
  try {
    torch cat [$t1 $t2] --dim 5
    error make {msg: "Expected error from invalid dimension"}
  } catch {
    # expected - dim 5 out of bounds for 2D tensor
  }
}

@test
def "Error case - negative dimension" [] {
  let input_data = $in
  let t1 = (torch full [2 3] 1)
  let t2 = (torch full [2 3] 2)
  try {
    torch cat [$t1 $t2] --dim (-1)
    error make {msg: "Expected error from negative dimension"}
  } catch {
    # expected - dimension must be non-negative
  }
}

@test
def "Error case - different number of dimensions" [] {
  let input_data = $in
  let t1 = (torch full [2 3] 1)
  let t2 = (torch full [2 3 4] 2)
  try {
    torch cat [$t1 $t2]
    error make {msg: "Expected error from different dimensions"}
  } catch {
    # expected - 2D vs 3D tensors
  }
}
