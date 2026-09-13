use torch

# Run with the development NuTorch shell, without importing the legacy module.
def identity [value] { $value }

let x = torch tensor [1 2 3] --requires-grad
let alias = (identity {nested: [$x]}).nested.0
print $x
print $"Native type: ($alias | describe)"
print $"Data: ($x | torch tolist | to json --raw)"

$x | torch mul $alias | torch sum | torch backward
let snapshot = ($alias | torch grad)
print $"Gradient through alias: ($snapshot | torch tolist | to json --raw)"

torch zero_grad $alias
print $"Cleared through original: ($x | torch grad | torch tolist | to json --raw)"
print $"Earlier snapshot: ($snapshot | torch tolist | to json --raw)"
print "Expected: gradient [2,4,6], cleared [0,0,0], snapshot still [2,4,6]."

# Source this file to keep $x in your interactive scope. Then test Ctrl+C with:
# loop { $x | torch mul $x | torch sum | torch backward; torch zero_grad $x }
# After Ctrl+C, run: $x | torch mul $x | torch tolist
