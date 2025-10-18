use std assert
use std/testing *

@test
def "Test squeeze with pipeline input" [] {
  let input_data = $in
  let t = torch full [1 2 3] 1
  let result = ($t | torch squeeze 0 | torch shape)
  # [1, 2, 3] squeezed at dim 0 becomes [2, 3]
  assert ($result == [2 3])
}


@test
def "Test squeeze dimension 1" [] {
  let input_data = $in
  let t = torch full [2 1 3] 1
  let result = ($t | torch squeeze 1 | torch shape)
  # [2, 1, 3] squeezed at dim 1 becomes [2, 3]
  assert ($result == [2 3])
}

@test
def "Test squeeze dimension 2" [] {
  let input_data = $in
  let t = torch full [2 3 1] 1
  let result = ($t | torch squeeze 2 | torch shape)
  # [2, 3, 1] squeezed at dim 2 becomes [2, 3]
  assert ($result == [2 3])
}

@test
def "Test squeeze middle dimension" [] {
  let input_data = $in
  let t = torch full [2 1 3 4] 1
  let result = ($t | torch squeeze 1 | torch shape)
  # [2, 1, 3, 4] squeezed at dim 1 becomes [2, 3, 4]
  assert ($result == [2 3 4])
}

@test
def "Test squeeze preserves values" [] {
  let input_data = $in
  let t = torch tensor [[[1 2 3]]]
  let result = ($t | torch squeeze 0 | torch value)
  # Values should be preserved after squeeze
  assert ($result == [[1 2 3]])
}

@test
def "Test squeeze multiple times" [] {
  let input_data = $in
  let t = torch full [1 1 2 3] 1
  let result = ($t | torch squeeze 0 | torch squeeze 0 | torch shape)
  # [1, 1, 2, 3] -> [1, 2, 3] -> [2, 3]
  assert ($result == [2 3])
}

@test
def "Test squeeze to scalar" [] {
  let input_data = $in
  let t = torch full [1 1 1] 5
  let result = ($t | torch squeeze 0 | torch squeeze 0 | torch squeeze 0 | torch value)
  # Should end up with scalar value
  assert ($result == 5)
}

@test
def "Test squeeze row vector" [] {
  let input_data = $in
  let t = torch tensor [[1 2 3]]
  let t_unsqueezed = $t | torch unsqueeze 0
  let result = ($t_unsqueezed | torch squeeze 0 | torch shape)
  # Adding and removing dimension 0
  assert ($result == [1 3])
}

@test
def "Error case with invalid tensor ID" [] {
  let input_data = $in
  try {
    "invalid-uuid" | torch squeeze 0
    error make {msg: "Expected error from invalid tensor ID"}
  } catch {
    # expected
  }
}

@test
def "Error case with invalid dimension" [] {
  let input_data = $in
  try {
    let t = torch full [1 2 3] 1
    $t | torch squeeze 5
    error make {msg: "Expected error from invalid dimension"}
  } catch {
    # expected
  }
}

@test
def "Error case with negative dimension" [] {
  let input_data = $in
  try {
    let t = torch full [1 2 3] 1
    $t | torch squeeze (-1)
    error make {msg: "Expected error from negative dimension"}
  } catch {
    # expected
  }
}

@test
def "Error case with non-1 size dimension" [] {
  let input_data = $in
  try {
    let t = torch full [2 3 4] 1
    $t | torch squeeze 0
    error make {msg: "Expected error from squeezing non-1 dimension"}
  } catch {
    # expected
  }
}

@test
def "Error case with dimension size 2" [] {
  let input_data = $in
  try {
    let t = torch full [1 2 3] 1
    $t | torch squeeze 1
    error make {msg: "Expected error from squeezing dimension with size 2"}
  } catch {
    # expected
  }
}
