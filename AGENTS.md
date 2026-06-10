# Nutorch

**Nutorch is a proof-of-concept Nushell plugin** that demonstrates how to bring
PyTorch tensor operations to the command line by wrapping tch-rs (Rust bindings
for LibTorch, PyTorch's C++ backend).

[Agent development guide](https://agents.md/). `CLAUDE.md` is a symlink to this
file — Claude, Codex, and any other agent read the same contract.

## Rules

Do exactly what your user says. No more, no less. NEVER assume they want
something they didn't ask for. NEVER change code unless explicitly asked.

When editing Rust code, always run `cargo fmt`. Accept the formatter output as
the source of truth. Do not manually undo, minimize, or selectively revert
`cargo fmt` formatting changes, including import ordering or wrapping changes.

Markdown, TOML, and JSON files are formatted with dprint (`dprint fmt`, config
in `dprint.json`). Accept the formatter output as the source of truth.

## Project Overview

**This is NOT production-ready software.** It is an experimental project
exploring the intersection of shell scripting and deep learning, with the goal
of making GPU-accelerated tensor operations available directly in the terminal.

### Project Status & Quality Tracking

**See [TODO.md](TODO.md) for complete implementation status and quality
metrics.**

The TODO.md file tracks:

- **Implementation completeness**: Which PyTorch methods are implemented
  (39/200+)
- **Quality criteria**: Test coverage, helper usage, validation, etc. for each
  method
- **Progress to 1.0**: What needs to be done for each method to be
  production-ready
- **Future roadmap**: PyTorch methods not yet implemented

When working on this project, **always consult TODO.md** to:

1. Check which methods need quality improvements
2. See what tests are missing
3. Understand which patterns need to be applied consistently
4. Track overall project completeness

### The Wrapping Layers

```
Nushell (Shell)
    ↓ nu-plugin protocol (MsgPack)
Nutorch Plugin (Rust)
    ↓ tch-rs bindings
LibTorch (C++)
    ↓ native CUDA/MPS/CPU
Hardware (GPU/CPU)
```

Each layer adds abstraction:

- **Nushell**: Shell with structured data (tables, lists, records)
- **Nutorch**: Plugin converting between Nushell values and tensors
- **tch-rs**: Safe Rust API over unsafe C++ FFI
- **LibTorch**: PyTorch's production C++ library
- **Hardware**: Actual compute devices

## Counter-Intuitive Facts

### 1. **Tensors Never Leave Rust**

Despite Nushell being a data-processing shell, **actual tensor data never
crosses the plugin boundary**. Commands pass UUID strings (like
`"a3f2e8c1-..."`) through pipelines, not tensors.

```nushell
# This looks like it passes a tensor through the pipe
let $x = [1 2 3] | torch tensor

# But $x is actually just a string UUID
$x | describe  # → "string"
```

**Why?** Nushell can only serialize simple types (strings, numbers, lists,
records) via MsgPack. Multi-dimensional arrays with GPU backing don't fit this
model.

### 2. **The Registry is a Memory Leak Risk**

The `TENSOR_REGISTRY` is a global HashMap that never automatically clears
(except via garbage collection). Every tensor operation creates a new UUID
entry:

```nushell
# Each intermediate result creates a registry entry
torch full [1000 1000] 1 | torch mm (torch full [1000 1000] 1) | torch mean
# → 3 tensors stored in registry, only the final ID returned
```

**Mitigation**: Nushell's plugin garbage collection (default: 10 seconds) kills
the plugin process, clearing the registry. Users should configure longer GC
intervals:

```nushell
$env.config.plugin_gc = {
  plugins: {
    nutorch: {
      stop_after: 10min  # Prevent premature tensor deletion
    }
  }
}
```

### 3. **In-Place Operations Aren't Really In-Place**

PyTorch has in-place operations (e.g., `tensor.add_(1)`), but Nutorch can't
expose them idiomatically because:

- Nushell is immutable-by-default
- UUID strings are the interface, not tensor references
- `torch sgd_step` appears to mutate, but actually does internal in-place
  updates while returning the same UUID

```nushell
let $w = torch full [2 2] 1 --requires_grad true
# This returns the same UUID, but modifies the underlying tensor
[$w] | torch sgd_step --lr 0.1
```

**Why it works**: The UUID still points to the same PyTorch tensor object, which
was mutated via `f_sub_()` in Rust.

### 4. **Device Placement is Critical and Manual**

Unlike PyTorch which can auto-transfer tensors, Nutorch **requires explicit
device matching**:

```nushell
# This FAILS - device mismatch
let $cpu_tensor = torch full [2 2] 1
let $gpu_tensor = torch full [2 2] 1 --device mps
$cpu_tensor | torch add $gpu_tensor  # ❌ Error from LibTorch

# Must manually ensure both tensors on same device
let $a = torch full [2 2] 1 --device mps
let $b = torch full [2 2] 1 --device mps
$a | torch add $b  # ✅ Works
```

### 5. **Gradients are Stored in PyTorch's Graph, Not Registry**

The `torch grad` command returns a _new_ UUID pointing to the gradient tensor,
but the gradient itself lives in PyTorch's autograd graph attached to the
original tensor.

```nushell
let $w = torch full [1] 2 --requires_grad true
let $loss = $w | torch mean
$loss | torch backward

# This creates a NEW registry entry for the gradient
let $grad_id = $w | torch grad
# The gradient tensor is cloned from PyTorch's graph into registry
```

### 6. **Binary Name ≠ Package Name**

The Cargo.toml specifies:

```toml
[package]
name = "nutorch" # Crate name

[[bin]]
name = "nu_plugin_torch" # Binary name (Nushell convention)
```

This means:

- `cargo install nutorch` installs the package
- But the binary is `~/.cargo/bin/nu_plugin_torch`
- And Nushell commands use `torch` namespace (not `nutorch`)

### 7. **Tests Use NPM, Not Cargo**

Despite being a Rust project, the test suite uses Nushell's testing framework
via NPM:

```bash
cd cargo/test
pnpm install  # Installs test.nu framework
nu -c "use node_modules/test.nu; test run-tests"
```

This is because tests verify **Nushell command behavior**, not Rust internals.

### 8. **Shape Validation Happens in Rust, Not PyTorch**

Many commands pre-validate tensor shapes before calling tch-rs:

```rust
// command_gather.rs validates dimensions before tch call
if dim < 0 || dim >= tensor_rank {
    return Err(LabeledError::new("Invalid dimension")...);
}
```

**Why?** tch-rs errors from C++ are opaque and crash-prone. Rust-side validation
provides better error messages.

### 9. **Dual Input Isn't Optional - It's Core Identity**

Every command implements dual input not for convenience, but because it's **how
the library bridges PyTorch and Nushell paradigms**. When implementing new
commands:

❌ **Wrong mindset**: "Should I add dual input support?" ✅ **Correct mindset**:
"Which dual input pattern does this operation type require?"

**Operation Type Requirements**:

- **Binary ops** (add, mul, mm): MUST support both `$t1 | torch op $t2` AND
  `torch op $t1 $t2`
- **Unary ops** (sin, mean, shape): MUST support both `$t | torch op` AND
  `torch op $t`
- **List ops** (cat, stack): MUST support both `[$ts] | torch op` AND
  `torch op [$ts]`
- **Utilities** (manual_seed, devices): Context-dependent, often args-only

**Missing dual input support is like missing a required parameter** - it breaks
the API contract and makes the command feel inconsistent with the rest of the
library.

## File Structure

```
nutorch/
├── README.md                    # User-facing documentation
├── LICENSE                      # Apache 2.0
├── AGENTS.md                    # This file (agent contract + architecture)
├── CLAUDE.md                    # Symlink to AGENTS.md
├── TODO.md                      # Implementation status and quality tracking
├── dprint.json                  # Code formatter config
│
├── issues/                      # ⭐ Issues and Experiments (the workflow)
│   ├── README.md                # Generated index (scripts/build-issues-index.sh)
│   └── {NNNN}-{slug}/           # One folder per issue
│       ├── README.md            # Issue spine: frontmatter, goal, experiments index
│       └── NN-{slug}.md         # One file per experiment
│
├── skills/                      # Agent skills (symlinked from .claude/skills)
│   ├── adversarial-review/      # In-session fresh-context review subagent
│   ├── claude-review/           # External claude -p reviewer with session log
│   └── create-skill/            # Meta-skill for authoring new skills
│
├── scripts/
│   └── build-issues-index.sh    # Regenerate issues/README.md
│
├── .claude/
│   ├── skills -> ../skills      # Symlink
│   └── agents/
│       └── adversarial-reviewer.md  # Named reviewer subagent definition
│
├── cargo/                       # ⭐ Main Rust plugin implementation
│   ├── Cargo.toml               # Package: "nutorch", Binary: "nu_plugin_torch"
│   ├── README.md                # Build/development instructions
│   │
│   ├── src/
│   │   ├── main.rs              # Plugin entry point (3 lines)
│   │   ├── lib.rs               # Core: registry, conversions, utilities
│   │   ├── command_*.rs         # 37 command implementations (one per file)
│   │   └── ...                  # Each command is ~100-200 lines
│   │
│   ├── test/
│   │   ├── package.json         # NPM config for test.nu framework
│   │   ├── test_*.nu            # 26 Nushell test files
│   │   └── node_modules/        # test.nu testing framework
│   │
│   ├── chat.md                  # Development history (superseded by issues/)
│   └── chat.archive.*.md        # Historical development discussions
│
├── npm/                         # Nushell-based utilities (higher-level)
│   │
│   ├── nn.nu/                   # Neural network helpers
│   │   ├── package.json         # NPM package metadata
│   │   ├── pyproject.toml       # Python reference implementation metadata
│   │   ├── README.md            # Usage instructions
│   │   ├── mod.nu               # Main module file
│   │   ├── *.nu                 # Training loop helpers, loss functions
│   │   ├── *.py                 # Reference Python implementations
│   │   └── chat.*.md            # Development discussions
│   │
│   └── beautiful.nu/            # Data visualization utilities
│       ├── package.json
│       ├── README.md
│       └── *.nu
│
├── demo/                        # Example usage scripts (untracked)
│   └── *.json                   # Test data files
│
├── logs/                        # Review logs and scratch output (gitignored)
│
├── raw-images/                  # Screenshots for README
│   └── *.png
│
└── chat.*.md                    # Root-level development discussions (4 archives)
```

### Key File Patterns

#### **Command Implementation Pattern** (`cargo/src/command_*.rs`)

Every command follows this exact structure:

```rust
use nu_plugin::PluginCommand;
use nu_protocol::{Category, Example, LabeledError, PipelineData, Signature, ...};
use crate::{NutorchPlugin, TENSOR_REGISTRY, ...};

pub struct CommandXxx;

impl PluginCommand for CommandXxx {
    type Plugin = NutorchPlugin;

    fn name(&self) -> &str { "torch xxx" }
    fn description(&self) -> &str { "..." }
    fn signature(&self) -> Signature {
        Signature::build("torch xxx")
            .input_output_types(vec![...])
            .optional/required/named(...)
            .category(Category::Custom("torch".into()))
    }
    fn examples(&self) -> Vec<Example> { vec![...] }
    fn run(&self, ..., input: PipelineData) -> Result<PipelineData, LabeledError> {
        // 1. Extract input (pipeline XOR argument)
        // 2. Fetch tensor(s) from TENSOR_REGISTRY
        // 3. Perform operation via tch-rs
        // 4. Store result in TENSOR_REGISTRY with new UUID
        // 5. Return UUID as string in PipelineData
    }
}
```

**Naming convention**:

- Struct: `CommandXxx` (PascalCase)
- Command: `torch xxx` (lowercase with spaces)
- File: `command_xxx.rs` (snake_case)

#### **Test Pattern** (`cargo/test/test_*.nu`)

```nushell
use std assert
use std/testing *

@test
def "Test xxx scenario 1" [] {
  let $t = torch xxx [args]
  # Perform operations
  let $result = $t | torch value
  assert ($result == expected)
}

@test
def "Test xxx scenario 2" [] {
  # Another test case
}
```

**Pattern**: Multiple `@test` functions per file, each testing a specific
scenario.

#### **Chat History Pattern (superseded)**

- `chat.md`: Work-in-progress discussions (historical)
- `chat.archive.N.md`: Archived discussions (numbered sequentially)
- Each subdirectory (`cargo/`, `npm/nn.nu/`) has its own chat files

The chat files are historical records from before the Issues and Experiments
workflow existed. Do not add to them — new work is recorded in `issues/`
instead. Do not modify or delete the archives.

## Architecture Deep Dive

### The Tensor Registry (`lib.rs:88-91`)

```rust
lazy_static! {
    pub static ref TENSOR_REGISTRY: Mutex<HashMap<String, Tensor>>
        = Mutex::new(HashMap::new());
}
```

**Critical properties**:

- **Global**: Single process-wide registry
- **Thread-safe**: `Mutex` guards concurrent access (though Nushell plugins are
  single-threaded)
- **String keys**: UUIDs from `uuid::Uuid::new_v4()`
- **Tensor values**: `tch::Tensor` (reference-counted internally by LibTorch)

**Lifecycle**:

1. Command creates tensor via tch-rs
2. UUID generated, tensor inserted: `registry.insert(uuid.clone(), tensor)`
3. UUID returned to Nushell as string
4. Subsequent commands look up tensor: `registry.get(&uuid)`
5. Plugin exit clears entire registry (Nushell GC kills process)

### Data Conversion (`lib.rs:198-336`)

#### **Nushell → Tensor** (`value_to_tensor`)

Handles recursive conversion:

```rust
Value::List → Tensor
  - Flat list [1,2,3] → 1D tensor
  - Nested [[1,2],[3,4]] → 2D tensor (recursive stacking)
  - Deep nesting [[[...]]] → ND tensor
Value::Int/Float → 0D (scalar) tensor
```

**Auto type inference**: All-integer lists → `Int64`, any float → `Float`

#### **Tensor → Nushell** (`tensor_to_value`)

Inverse operation:

```rust
0D tensor → Value::Int or Value::Float
1D tensor → Value::List (flat)
ND tensor → Value::List (nested, recursive)
```

**Element extraction**: Uses `tensor.get(i)` in loop (no direct `to_vec()` in
tch-rs)

### Dual Input Pattern (CORE DESIGN PRINCIPLE)

**This is not just a feature - it's the fundamental design principle that makes
Nutorch feel native to both ecosystems.**

#### Why This Matters

PyTorch has two API styles:

- **Method form**: `tensor1.add(tensor2)` - object-oriented
- **Function form**: `torch.add(tensor1, tensor2)` - functional

Nushell's paradigm is pipeline-based:

- Data flows left-to-right through transformations
- Commands consume stdin and produce stdout

**Nutorch bridges both worlds by supporting BOTH patterns:**

#### Binary Operations

**Every binary operation supports both**:

```nushell
# Pipeline + Argument (feels like tensor1.add(tensor2))
([1] | torch tensor) | torch add ([2] | torch tensor)

# Two Arguments (feels like torch.add(tensor1, tensor2))
torch add ([1] | torch tensor) ([2] | torch tensor)
```

**Python PyTorch equivalents**:

```python
# Method form
torch.tensor([1]).add(torch.tensor([2]))
# or with operator
torch.tensor([1]) + torch.tensor([2])

# Function form
torch.add(torch.tensor([1]), torch.tensor([2]))
```

#### Unary Operations

```nushell
# Pipeline (feels like tensor.sin())
$tensor | torch sin

# Argument (feels like torch.sin(tensor))
torch sin $tensor
```

#### List Operations

```nushell
# Pipeline
[$t1 $t2] | torch cat --dim 0

# Argument
torch cat [$t1 $t2] --dim 0
```

**Implementation** (standard across all commands):

```rust
let piped = match input {
    PipelineData::Value(v, _) => Some(v),
    PipelineData::Empty => None,
    _ => return Err(...),
};
let arg0 = call.nth(0);

match (&piped, &arg0) {
    (None, None) => Err("Missing input"),
    (Some(_), Some(_)) => Err("Conflicting input"),  // XOR enforcement
    _ => Ok(...)
}
```

**Benefits**:

1. **PyTorch users** can write familiar imperative code
2. **Nushell users** can build natural pipelines
3. **Complex expressions** remain readable in either style
4. **Gradual learning** - start imperative, adopt pipelines as comfortable

**Example of mixed style**:

```nushell
# Readable complex expression mixing both patterns
let $result = torch softmax (torch add ($x | torch mm $w) $b) --dim 1
```

### Autograd Implementation

#### **Forward Pass**

```nushell
let $x = torch randn [5 3] --requires_grad true
let $w = torch randn [3 2] --requires_grad true
let $y = $x | torch mm $w
```

**Behind the scenes**:

- `--requires_grad true` → `tensor.set_requires_grad(true)` in Rust
- PyTorch builds computation graph automatically
- Each operation creates new tensor with gradient function attached

#### **Backward Pass** (`command_backward.rs`)

```nushell
$loss | torch backward
```

**Implementation**:

1. Validates `loss.numel() == 1` (scalar only)
2. Calls `loss.backward()` (PyTorch computes all gradients)
3. Gradients stored in autograd graph, not registry

#### **Gradient Access** (`command_grad.rs`)

```nushell
let $grad = $w | torch grad
```

**Implementation**:

1. Fetch tensor from registry
2. Call `tensor.grad()` (returns `Tensor`, possibly undefined)
3. Check `grad.defined()` → return `null` if false
4. **Clone gradient into registry** with new UUID
5. Return gradient UUID

**Quirk**: Gradients are cloned/copied into registry, not referenced.

#### **Optimizer** (`command_sgd_step.rs`)

```nushell
[$w1 $w2] | torch sgd_step --lr 0.01
```

**Implementation**:

1. Accepts **list** of parameter UUIDs
2. Fetches all tensors from registry
3. **In-place update** within `tch::no_grad()`:
   ```rust
   p.f_sub_(&(p.grad() * lr))  // p -= lr * grad
   ```
4. Returns **same UUIDs** (tensor objects mutated, not replaced)

## Development Workflow

### Building

```bash
cd cargo
cargo build --release
# Binary: cargo/target/release/nu_plugin_torch
```

### Testing

**Important**: Tests must be run from the `cargo/test` directory using the
`test.nu` framework.

```bash
cd cargo/test
pnpm install  # First time only - installs test.nu framework

# Then from within Nushell:
use node_modules/test.nu
test run-tests  # Runs all test_*.nu files
```

**Step-by-step**:

1. `cd cargo/test` - Navigate to test directory
2. `pnpm install` - Install test.nu (one time only)
3. `nu` - Start Nushell
4. `use node_modules/test.nu` - Load testing framework
5. `test run-tests` - Execute all tests

This will run all `test_*.nu` files in the directory.

### Installing Locally

```bash
# From cargo/ directory
cargo build --release
plugin add target/release/nu_plugin_torch
plugin use torch
```

### Environment Variables (macOS with Homebrew Python)

```nushell
$env.LIBTORCH = "/opt/homebrew/lib/python3.11/site-packages/torch"
$env.LD_LIBRARY_PATH = ($env.LIBTORCH | path join "lib")
$env.DYLD_LIBRARY_PATH = ($env.LIBTORCH | path join "lib")
```

**Why needed?** LibTorch is a dynamic library; Rust needs to find it at compile
and runtime.

## Command Categories (37 total)

### Tensor Creation (6)

- `torch tensor` - From Nushell lists
- `torch full` - Filled with value
- `torch randn` - Random normal
- `torch linspace` - Linear spacing
- `torch arange` - Range with step
- _(zeros/ones via `torch full 0/1`)_

### Element-wise Binary Ops (5)

- `torch add` (with `--alpha` for scaled addition)
- `torch sub`
- `torch mul`
- `torch div`
- `torch maximum` - Element-wise max of two tensors

### Element-wise Unary Ops (7)

- `torch neg`
- `torch sin`
- `torch exp`
- `torch softmax`
- `torch log_softmax`
- `torch detach` - Detach from autograd graph
- _(see lib.rs for complete list)_

### Reduction Ops (3)

- `torch mean`
- `torch max` - Return max value
- `torch argmax` - Return index of max

### Matrix Ops (2)

- `torch mm` - Matrix multiplication
- `torch t` - Transpose (2D only)

### Shape Manipulation (7)

- `torch shape` - Get shape as list
- `torch squeeze` - Remove size-1 dims
- `torch unsqueeze` - Add size-1 dim
- `torch reshape` - Change shape
- `torch cat` - Concatenate along existing dim
- `torch stack` - Stack along new dim
- `torch repeat` - Repeat tensor
- `torch repeat_interleave` - Repeat elements

### Indexing (1)

- `torch gather` - Advanced indexing (for loss functions)

### Autograd (4)

- `torch backward` - Backpropagation
- `torch grad` - Access gradients
- `torch zero_grad` - Clear gradients
- `torch detach` - Stop gradient tracking

### Optimization (1)

- `torch sgd_step` - Vanilla SGD update

### Utilities (4)

- `torch value` - Tensor → Nushell
- `torch free` - Remove from registry
- `torch devices` - List available devices
- `torch manual_seed` - Set random seed

## Common Flags

**All tensor creation commands**:

- `--device <cpu|cuda|cuda:N|mps>` (default: cpu)
- `--dtype <float32|float64|int32|int64>` (default: float32)
- `--requires_grad <bool>` (default: false)

**Many operations**:

- `--dim <int>` - Dimension to operate along

## Known Limitations

1. **No automatic broadcasting** - Must manually match shapes
2. **No operator overloading** - Can't use `$a + $b`, must use `torch add`
3. **No Python-style slicing** - Must use specific commands
4. **No model serialization** - Can't save/load trained models yet
5. **macOS-only tested** - Windows/Linux may need customization
6. **Limited activation functions** - Only sin, softmax, log_softmax
7. **No optimization state** - SGD only, no Adam/momentum
8. **No conv/pooling layers** - Linear operations only

## Future Directions (from README TODO)

- [ ] Implement remaining PyTorch tensor operations
- [ ] Add `tch::nn` module wrapper (layers, optimizers)
- [ ] Model serialization (save/load)
- [ ] More optimizers (Adam, RMSprop)
- [ ] Convolutional and pooling operations
- [ ] Better dimension validation across all commands
- [ ] Windows/Linux compatibility testing

## Design Philosophy

1. **PyTorch API compatibility** - Command names and argument order match
   PyTorch where possible
2. **Nushell idioms** - Pipeline-friendly, structured data output
3. **Dual Input Pattern - CORE PRINCIPLE** - Every command mirrors PyTorch's
   method/function duality while embracing Nushell's pipeline philosophy:
   - Binary ops: `$t1 | torch add $t2` OR `torch add $t1 $t2`
   - Unary ops: `$t | torch sin` OR `torch sin $t`
   - Enables both imperative (Python-like) and functional (Nushell-like) styles
   - Users can compose pipelines naturally while maintaining PyTorch familiarity
   - **This is not optional** - it's how the library bridges both ecosystems
4. **Explicit over implicit** - Manual device placement, no auto-casting
5. **Proof-of-concept first** - Focus on demonstrating feasibility, not
   completeness
6. **Shell-native deep learning** - Make GPU programming accessible from
   terminal

## Issues and Experiments

Every significant piece of work gets an issue in `issues/`. Issues describe the
problem, provide background, and propose solutions. Experiments are the
incremental steps that solve the problem.

### Issue Structure

Each issue is a **folder**. The `README.md` is the issue **spine** (frontmatter,
goal, background, analysis, an ordered index of experiments, and the final
conclusion). **Every experiment is its own numbered file** in the same folder —
the README never contains experiment bodies, only links to them.

```
issues/0001-nutorchd-architecture/
├── README.md                     ← spine: frontmatter, goal, background,
│                                    the ordered Experiments index, conclusion
├── 01-stand-up-daemon.md         ← Experiment 1 (full body in its own file)
├── 02-wire-first-op.md           ← Experiment 2
└── 03-...                        ← one file per experiment, in sequence
```

The folder name is `{NNNN}-{slug}`. The number is zero-padded to 4 digits and
globally sequential across the whole project. The slug is lowercase, hyphenated,
and describes the topic.

**Why one file per experiment:** it keeps experiments ordered and easy to read,
access, and organize (up to ~100 per issue with clean `NN-` filenames), and —
critically — it makes experiments easy to **automate**: each experiment is a
discrete file created and tracked from the README, rather than ever-growing
edits to one monolithic document.

The full index of all issues is at `issues/README.md`. Regenerate it with:

```bash
scripts/build-issues-index.sh
```

#### Frontmatter

Every `README.md` starts with TOML frontmatter:

```
+++
status = "open"
opened = "2026-06-10"
+++
```

Or for closed issues:

```
+++
status = "closed"
opened = "2026-06-10"
closed = "2026-06-10"
+++
```

Issues may add their own TOML frontmatter keys — to `README.md`, experiment
files, or other issue docs — for issue-specific metadata such as per-experiment
agent provenance, as long as:

- the reserved workflow keys are preserved: `README.md` always carries `status`
  and `opened` (plus `closed` when closed), unchanged in name and meaning;
- additive keys are valid TOML between the `+++` delimiters and do not
  contradict the reserved keys or the index tooling —
  `scripts/build-issues-index.sh` reads only the reserved README keys and
  ignores the rest;
- the issue documents its own added schema in its `README.md`.

#### README.md structure

After the frontmatter, a new issue's `README.md` has these sections:

1. **Title** (H1) — `# Issue {N}: {descriptive title}`
2. **Goal** — One or two sentences describing the desired outcome.
3. **Background** — Context, prior work, constraints.
4. **Architecture** / **Analysis** / **Proposed Solutions** — Technical details.

A new issue's README has **no experiments listed yet**.

As experiments are created, the README grows an **`## Experiments`** section: an
ordered list linking to each experiment file, one per line, with a one-line
status. The README holds the links and statuses only — never the experiment
bodies. Example:

```markdown
## Experiments

- [Experiment 1: Stand up the daemon](01-stand-up-daemon.md) — **Pass**
- [Experiment 2: Wire the first op](02-wire-first-op.md) — **Partial** (needs a
  length-prefixed framing fix)
- [Experiment 3: …](03-….md) — **Designed**
```

Keep each status to one of: `Designed`, `In progress`, `Pass`, `Partial`,
`Fail`. Update the line when the experiment's result is recorded, so the README
doubles as an at-a-glance progress tracker.

When the issue is solved or abandoned, add the **`## Conclusion`** section to
the README (see "Closing an Issue").

#### Experiment files

Each experiment lives in its **own file** `NN-{slug}.md` in the issue folder,
where `NN` is a zero-padded two-digit number in creation order (`01`, `02`, …,
up to `99`). The slug is lowercase-hyphenated and describes the experiment.

An experiment file may begin with an optional TOML frontmatter block
(`+++ … +++`) before its H1 title — for issue-specific metadata such as agent
provenance. Experiment frontmatter is optional and must not replace the required
H1 title and H2 sections below it.

Each experiment file contains:

1. **Title** (H1) — `# Experiment {N}: {descriptive title}`
2. **Description** — What and why.
3. **Changes** — Specific code changes, listed by file.
4. **Verification** — How to test. Concrete steps and pass/fail criteria.
5. **Result** and **Conclusion** — added after the experiment runs (see
   "Recording results").

Keep each file focused; if one grows past ~1000 lines, that is a sign the
experiment is too big and should be split into the next numbered experiment.

### Multiple Open Issues

Multiple issues can be open at the same time. This allows interleaving work — a
large issue like the nutorchd daemon can stay open while smaller issues are
opened and closed alongside it.

### Experiments

#### When to create an experiment

Only after the issue's requirements are clear. Each experiment is designed,
implemented, and concluded before the next one is designed.

**Never list experiments upfront.** The outcome of each experiment informs what
comes next.

#### Experiment structure

Each experiment is its own file `NN-{slug}.md` (see "Experiment files" above),
and is added as a new link in the README's `## Experiments` index the moment it
is created. Inside the file, use an H1 title and H2 sections:

1. **Title** (H1) — `# Experiment {N}: {descriptive title}`
2. **Description** (H2) — What and why.
3. **Changes** (H2) — Specific code changes, listed by file.
4. **Verification** (H2) — How to test. Concrete steps and pass/fail criteria.
5. **Result** / **Conclusion** (H2) — added after it runs.

#### Verification gates

Nutorch's standard hygiene checks for any experiment that touches code:

- `cargo build --release` (from `cargo/`) — builds with no new warnings;
- `cargo fmt -- --check` clean (from `cargo/`);
- the Nushell test suite green: from `cargo/test`,
  `nu -c "use node_modules/test.nu; test run-tests"`;
- `dprint check` clean for any touched markdown/TOML/JSON;
- new or changed `torch` commands honor the **dual input pattern** and match
  PyTorch's API (names, argument order, semantics) wherever possible;
- TODO.md quality criteria updated when a command's status changes.

An experiment's Verification section must state which of these apply plus any
experiment-specific checks (e.g. device-specific behavior on `mps`).

#### One at a time

Design and implement one experiment at a time. The result of Experiment 1
directly informs what Experiment 2 should be.

#### AI review gate

Every experiment must be reviewed by another AI agent before moving to the next
stage. For now the reviewer is **Claude reviewing Claude** — the in-session
`adversarial-reviewer` subagent (see the `adversarial-review` skill) or an
external `claude -p` process (see the `claude-review` skill). Cross-model
reviewers (Codex and others) will be added later; the gate itself does not
change.

1. **Design review before implementation**
   - After writing the experiment design, ask another AI agent to review it.
   - Fix all real issues found by the review.
   - Record the review result in the experiment file.
   - Do not implement the experiment until the reviewing agent approves the
     design.

2. **Result review before the next experiment**
   - After implementation, verification, and result recording, ask another AI
     agent to review the completed experiment and result.
   - Fix all real issues found by the review.
   - Record the completion-review result in the experiment file.
   - Do not design or implement the next experiment until the reviewing agent
     approves the completed output.

The reviewing agent must be separate from the implementation pass — at minimum a
fresh-context subagent; ideally (later) a different model entirely.

#### Experiment commits

Every experiment has two required commit points:

1. **Plan commit** — after the experiment design is written, reviewed, fixed,
   approved, and linked from the issue README, commit the experiment plan before
   implementation begins.
2. **Result commit** — after implementation, verification, result recording,
   completion review, and any required fixes, commit the experiment result
   before designing the next experiment.

These commits must be separate. Do not combine an experiment plan and its result
in the same commit, and do not start the next experiment before the previous
experiment's result commit exists.

#### Recording results

After testing, append the result **inside the experiment's own file**, below
Verification:

```markdown
## Result

**Result:** Pass / Partial / Fail

{description}

## Conclusion

{what we learned, what the next experiment should be}
```

Then update that experiment's status on its line in the README's
`## Experiments` index (`Designed` → `Pass`/`Partial`/`Fail`). All three
outcomes are valuable — failed experiments eliminate dead ends.

### Closing an Issue

When the issue is solved or abandoned, add a `## Conclusion` section to the
**`README.md`** (after the `## Experiments` index), summarizing what was learned
and the outcome. Update the frontmatter to `status = "closed"` with a `closed`
date. Regenerate the index:

```bash
scripts/build-issues-index.sh
```

### Immutability

Closed issues are historical records. They are **immutable** and must NEVER be
modified. History stays as it was written.

### Process Summary

1. **Create the issue** — `issues/{NNNN}-{slug}/README.md` with frontmatter,
   goal, background, analysis. No experiments yet.
2. **Design Experiment 1** — Create `01-{slug}.md` with the experiment body, and
   add a link to it under `## Experiments` in the README (status `Designed`).
3. **Review and commit the plan** — Get another AI agent to approve the design,
   fix real findings, record the review result, and commit the experiment plan.
4. **Implement Experiment 1** — Write the code.
5. **Record the result** — Append `## Result` / `## Conclusion` inside
   `01-{slug}.md`, and update its status on the README index line.
6. **Review and commit the result** — Get another AI agent to approve the
   completed output, fix real findings, record the completion review, and commit
   the experiment result.
7. **Repeat** — Create `02-{slug}.md` for the next experiment (the prior result
   informs it), link it from the README, and continue until the goal is met.
8. **Close the issue** — Write the `## Conclusion` in the README, update
   frontmatter, rebuild the index.

## Remember

NEVER change code unless explicitly asked. NEVER make unrequested changes.
Always do EXACTLY what your user asks — no more, no less.
