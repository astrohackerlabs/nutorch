---
title: Neural networks
description: Compose shared native modules, train with live optimizers and load compatible safetensors state files.
order: 6
section: Deep learning
---

Modules and optimizers are native values in the unreleased NuTorch shell.
Assignments share identity. Import the native commands with `use torch`;
no tensor server is required.

## Building modules

```nu
use torch
torch manual_seed 7
let first = torch nn linear 2 8
let model = torch nn sequential $first (torch nn relu) (torch nn linear 8 2)
let parameters = (torch nn parameters $first)
torch nn info $model
```

Sequential composition retains its children, so `$first` remains usable.
Parameters are live tensor views, including their gradients and later updates.
Explicit constructor weights and bias tensors are copied into new tracked leaves.

| Constructors | Main arguments |
| --- | --- |
| linear | input features, output features |
| relu, sigmoid, tanh, gelu | none |
| sequential | shared child modules |
| conv1d, conv2d, conv_transpose2d | input channels, output channels, kernel size |
| embedding | embedding count, embedding dimension |
| layer_norm | normalized-shape list |
| batch_norm | feature count |
| group_norm | group count, channel count |
| dropout | optional probability flag |
| leaky_relu | optional negative-slope flag |
| softmax | dimension |
| max_pool2d, avg_pool2d | kernel size |
| flatten | optional start/end dimensions |

Use `torch nn linear --help` or another constructor's help for its flags.
Underscore and hyphen spellings of constructor flags are accepted.
Duplicate children and overlapping subtrees are rejected.

## Training

Continue in the same NuTorch session with `$model` from **Building modules**.

```nu
use torch
let x = torch tensor [[0.0 0.0] [1.0 1.0]]
let labels = torch tensor [0 1] --dtype int64
let optimizer = torch nn adam $model --lr 0.05
for _ in 1..100 {
  $x | torch forward $model | torch cross_entropy $labels | torch backward
  torch step $optimizer
  torch nn zero_grad $optimizer
}
$x | torch forward $model | torch argmax --dim 1 | torch value
```

Expected predictions: `[0 1]`.

SGD, Adam, AdamW and RMSprop preserve the baseline options and update order.
SGD requires a learning rate. Optimizer aliases share state and learning rate;
`torch nn set_lr $optimizer 0.001` updates that shared rate.
Parameters without gradients are skipped. Optimizers retain their parameters
even after module variables leave scope.

`torch nn zero_grad` accepts a module or optimizer.
`torch nn train $model` and `torch nn eval $model` propagate through shared
children; dropout and batch normalization observe those modes.

## State files

Continue after **Training**. Save to a temporary file, load into a fresh model of
the same architecture, and compare predictions before deleting the temporary file:

```nu
use torch
let checkpoint = (mktemp --suffix .safetensors)
torch nn save $model $checkpoint
let restored = torch nn sequential (torch nn linear 2 8) (torch nn relu) (torch nn linear 8 2)
torch nn load $restored $checkpoint
print ($x | torch forward $restored | torch allclose ($x | torch forward $model))
rm $checkpoint
```

The prediction comparison returns `true`.

The file format remains compatible with baseline safetensors module state_dict
files. Names follow child-index qualification and include batch-normalization
buffers. All keys and shapes are validated before copying into existing storage.
Parameter aliases and already-created optimizers remain attached after loading.
Relative paths use the shell's current directory.

These files contain module state, not architecture or optimizer checkpoints.
For complete runnable examples, use the development checkout's
`code/nutorch/examples/train-regression.nu` and `train-classify.nu`.
