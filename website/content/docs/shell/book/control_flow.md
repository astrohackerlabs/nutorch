---
title: "Control Flow"
description: "Control Flow"
order: 1020
section: "Book · Programming in Nu"
---

Nushell provides several commands that help determine how different groups of code are executed. In programming languages this functionality is often referred to as _control flow_.

> **tip**
> One thing to note is that all of the commands discussed on this page use [blocks](/docs/shell/book/types_of_data/#blocks). This means you can mutate [environmental variables](/docs/shell/book/environment/) and other [mutable variables](/docs/shell/book/variables/#mutable-variables) in them.


## Already covered

Below we cover some commands related to control flow, but before we get to them, it's worthwhile to note there are several pieces of functionality and concepts that have already been covered in other sections that are also related to control flow or that can be used in the same situations. These include:

- Pipes on the [pipelines](/docs/shell/book/pipelines/) page.
- Closures on the [types of data](/docs/shell/book/types_of_data/) page.
- Iteration commands on the [working with lists](/docs/shell/book/working_with_lists/) page. Such as:
  - [`each`](/docs/shell/commands/each/)
  - [`where`](/docs/shell/commands/where/)
  - [`reduce`](/docs/shell/commands/reduce/)

## Choice (Conditionals)

The following commands execute code based on some given condition.

> **tip**
> The choice/conditional commands are expressions so they return values, unlike the other commands on this page. This means the following works.
> 
> ```nu
> 'foo' | if $in == 'foo' { 1 } else { 0 } | $in + 2
> # => 3
> ```
> 


### `if`

[`if`](/docs/shell/commands/if/) evaluates branching [blocks](/docs/shell/book/types_of_data/#blocks) of code based on the results of one or more conditions similar to the "if" functionality in other programming languages. For example:

```nu
if $x > 0 { 'positive' }
```

Returns `'positive`' when the condition is `true` (`$x` is greater than zero) and `null` when the condition is `false` (`$x` is less than or equal to zero).

We can add an `else` branch to the `if` after the first block which executes and returns the resulting value from the `else` block when the condition is `false`. For example:

```nu
if $x > 0 { 'positive' } else { 'non-positive' }
```

This time it returns `'positive'` when the condition is `true` (`$x` is greater than zero) and `'non-positive`' when the condition is `false` (`$x` is less than or equal to zero).

We can also chain multiple `if`s together like the following:

```nu
if $x > 0 { 'positive' } else if $x == 0 { 'zero' } else { "negative" }
```

When the first condition is `true` (`$x` is greater than zero) it will return `'positive'`, when the first condition is `false` and the next condition is `true` (`$x` equals zero) it will return `'zero'`, otherwise it will return `'negative'` (when `$x` is less than zero).

### `match`

[`match`](/docs/shell/commands/match/) executes one of several conditional branches based on the value given to match. You can also do some [pattern matching](/docs/shell/cookbook/pattern_matching/) to unpack values in composite types like lists and records.

Basic usage of [`match`](/docs/shell/commands/match/) can conditionally run different code like a "switch" statement common in other languages. [`match`](/docs/shell/commands/match/) checks if the value after the word [`match`](/docs/shell/commands/match/) is equal to the value at the start of each branch before the `=>` and if it does, it executes the code after that branch's `=>`.

```nu
match 3 {
    1 => 'one',
    2 => {
        let w = 'w'
        't' + $w + 'o'
    },
    3 => 'three',
    4 => 'four'
}
# => three
```

The branches can either return a single value or, as shown in the second branch, can return the results of a [block](/docs/shell/book/types_of_data/#blocks).

#### Catch all Branch

You can also have a catch all condition for when the given value doesn't match any of the other conditions by having a branch whose matching value is `_`.

```nu
let foo = match 7 {
    1 => 'one',
    2 => 'two',
    3 => 'three',
    _ => 'other number'
}
$foo
# => other number
```

(Reminder, [`match`](/docs/shell/commands/match/) is an expression which is why we can assign the result to `$foo` here).

#### Pattern Matching

You can "unpack" values from types like lists and records with [pattern matching](/docs/shell/cookbook/pattern_matching/). You can then assign variables to the parts you want to unpack and use them in the matched expressions.

```nu
let foo = { name: 'bar', count: 7 }
match $foo {
    { name: 'bar', count: $it } => ($it + 3),
    { name: _, count: $it } => ($it + 7),
    _ => 1
}
# => 10
```

The `_` in the second branch means it matches any record with field `name` and `count`, not just ones where `name` is `'bar'`.

#### Guards

You can also add an additional condition to each branch called a "guard" to determine if the branch should be matched. To do so, after the matched pattern put `if` and then the condition before the `=>`.

```nu
let foo = { name: 'bar', count: 7 }
match $foo {
    { name: 'bar', count: $it } if $it < 5 => ($it + 3),
    { name: 'bar', count: $it } if $it >= 5 => ($it + 7),
    _ => 1
}
# => 14
```

---

You can find more details about [`match`](/docs/shell/commands/match/) in the [pattern matching cookbook page](https://www.nushell.sh/cookbook/pattern_matching.html).

## Loops

The loop commands allow you to repeat a block of code multiple times.

### Loops and Other Iterating Commands

The functionality of the loop commands is similar to commands that apply a closure over elements in a list or table like [`each`](/docs/shell/commands/each/) or [`where`](/docs/shell/commands/where/) and many times you can accomplish the same thing with either. For example:

```nu
mut result = []
for $it in [1 2 3] { $result = ($result | append ($it + 1)) }
$result
# => ╭───┬───╮
# => │ 0 │ 2 │
# => │ 1 │ 3 │
# => │ 2 │ 4 │
# => ╰───┴───╯


[1 2 3] | each { $in + 1 }
# => ╭───┬───╮
# => │ 0 │ 2 │
# => │ 1 │ 3 │
# => │ 2 │ 4 │
# => ╰───┴───╯
```

While it may be tempting to use loops if you're familiar with them in other languages, it is considered more in the [Nushell-style](/docs/shell/book/thinking_in_nu/) (idiomatic) to use commands that apply closures when you can solve a problem either way. The reason for this is because of a pretty big downside with using loops.

#### Loop Disadvantages

The biggest downside of loops is that they are statements, unlike [`each`](/docs/shell/commands/each/) which is an expression. Expressions, like [`each`](/docs/shell/commands/each/) always result in some output value, however statements do not.

This means that they don't work well with immutable variables and using immutable variables is considered a more [Nushell-style](/docs/shell/book/thinking_in_nu/#variables-are-immutable-by-default). Without a mutable variable declared beforehand in the example in the previous section, it would be impossible to use [`for`](/docs/shell/commands/each/) to get the list of numbers with incremented numbers, or any value at all.

Statements also don't work in Nushell pipelines which require some output. In fact Nushell will give an error if you try:

```nu
[1 2 3] | for x in $in { $x + 1 } | $in ++ [5 6 7]
# => Error: nu::parser::unexpected_keyword
# => 
# =>   × Statement used in pipeline.
# =>    ╭─[entry #5:1:1]
# =>  1 │ [1 2 3] | for x in $in { $x + 1 } | $in ++ [5 6 7]
# =>    ·           ─┬─
# =>    ·            ╰── not allowed in pipeline
# =>    ╰────
# =>   help: 'for' keyword is not allowed in pipeline. Use 'for' by itself, outside of a pipeline.
```

Because Nushell is very pipeline oriented, this means using expression commands like [`each`](/docs/shell/commands/each/) is typically more natural than loop statements.

#### Loop Advantages

If loops have such a big disadvantage, why do they exist? Well, one reason is that closures, like [`each`](/docs/shell/commands/each/) uses, can't modify mutable variables in the surrounding environment. If you try to modify a mutable variable in a closure you will get an error:

```nu
mut foo = []
[1 2 3] | each { $foo = ($foo | append ($in + 1)) }
# => Error: nu::parser::expected_keyword
# => 
# =>   × Capture of mutable variable.
# =>    ╭─[entry #8:1:1]
# =>  1 │ [1 2 3] | each { $foo = ($foo | append ($in + 1)) }
# =>    ·                  ──┬─
# =>    ·                    ╰── capture of mutable variable
# =>    ╰────
```

If you modify an environmental variable in a closure, you can, but it will only modify it within the scope of the closure, leaving it unchanged everywhere else. Loops, however, use [blocks](/docs/shell/book/types_of_data/#blocks) which means they can modify a regular mutable variable or an environmental variable within the larger scope.

```nu
mut result = []
for $it in [1 2 3] { $result = ($result | append ($it + 1)) }
$result
# => ╭───┬───╮
# => │ 0 │ 2 │
# => │ 1 │ 3 │
# => │ 2 │ 4 │
# => ╰───┴───╯
```

When creating custom errors, loops have an advantage over pipelines because they don't fail evaluation.

Creating an error in a pipeline yields a pipeline evaluation error *and* the custom error:

```nu
[0] | each { error make { msg: "Custom error" } | default '' }
# => Error: nu::shell::eval_block_with_input
# => 
# =>   × Eval block failed with pipeline input
# =>    ╭─[entry #20:1:2]
# =>  1 │ [0] | each { error make { msg: "Custom error" } | default '' }
# =>    ·  ┬
# =>    ·  ╰── source value
# =>    ╰────
# => 
# => Error:
# =>   × Custom error
# =>    ╭─[entry #20:1:14]
# =>  1 │ [0] | each { error make { msg: "Custom error" } | default '' }
# =>    ·              ─────┬────
# =>    ·                   ╰── originates from here
# =>    ╰────
```

When using a loop, a `for` loop in this example, evaluation succeeds and only the created error occurs:

```nu
for x in [0] { error make { msg: "Custom error" } }
# => Error: nu::shell::error
# => 
# =>   × Custom error
# =>    ╭─[entry #18:1:16]
# =>  1 │ for x in [0] { error make { msg: "Custom error" } }
# =>    ·                ─────┬────
# =>    ·                     ╰── originates from here
# =>    ╰────
```

### `for`

[`for`](/docs/shell/commands/for/) loops over a range or collection like a list or a table.

```nu
for x in [1 2 3] { $x * $x | print }
# => 1
# => 4
# => 9
```

#### Expression Command Alternatives

- [`each`](/docs/shell/commands/each/)
- [`par-each`](/docs/shell/commands/par-each/)
- [`where`](/docs/shell/commands/where/)/[`filter`](/docs/shell/commands/filter/)
- [`reduce`](/docs/shell/commands/reduce/)

### `while`

[`while`](/docs/shell/commands/while/) loops the same block of code until the given condition is `false`.

```nu
mut x = 0; while $x < 10 { $x = $x + 1 }; $x
# => 10
```

#### Expression Command Alternatives

The "until" and other "while" commands

- [`take until`](/docs/shell/commands/take_until/)
- [`take while`](/docs/shell/commands/take_while/)
- [`skip until`](/docs/shell/commands/skip_until/)
- [`skip while`](/docs/shell/commands/skip_while/)

### `loop`

[`loop`](/docs/shell/commands/loop/) loops a block infinitely. You can use [`break`](/docs/shell/commands/break/) (as described in the next section) to limit how many times it loops. It can also be handy for continuously running scripts, like an interactive prompt.

```nu
mut x = 0; loop { if $x > 10 { break }; $x = $x + 1 }; $x
# => 11
```

### `break`

[`break`](/docs/shell/commands/break/) will stop executing the code in a loop and resume execution after the loop. Effectively "break"ing out of the loop.

```nu
for x in 1..10 { if $x > 3 { break }; print $x }
# => 1
# => 2
# => 3
```

### `continue`

[`continue`](/docs/shell/commands/continue/) will stop execution of the current loop, skipping the rest of the code in the loop, and will go to the next loop. If the loop would normally end, like if [`for`](/docs/shell/commands/for/) has iterated through all the given elements, or if [`while`](/docs/shell/commands/while/)'s condition is now false, it won't loop again and execution will continue after the loop block.

```nu
mut x = -1; while $x <= 6 { $x = $x + 1; if $x mod 3 == 0 { continue }; print $x }
# => 1
# => 2
# => 4
# => 5
# => 7
```

## Errors

### `error make`

[`error make`](/docs/shell/commands/error_make/) creates an error that stops execution of the code and any code that called it, until either it is handled by a [`try`](/docs/shell/commands/try/) block, or it ends the script and outputs the error message. This functionality is the same as "exceptions" in other languages.

```nu
print 'printed'; error make { msg: 'Some error info' }; print 'unprinted'
# => printed
# => Error:   × Some error info
# =>    ╭─[entry #9:1:1]
# =>  1 │ print 'printed'; error make { msg: 'Some error info' }; print 'unprinted'
# =>    ·                  ─────┬────
# =>    ·                       ╰── originates from here
# =>    ╰────
```

The record passed to it provides some information to the code that catches it or the resulting error message.

You can find more information about [`error make`](/docs/shell/commands/error_make/) and error concepts on the [Creating your own errors page](/docs/shell/book/creating_errors/).

### `try`

[`try`](/docs/shell/commands/try/) will catch errors created anywhere in the [`try`](/docs/shell/commands/try/)'s code block and resume execution of the code after the block.

```nu
try { error make { msg: 'Some error info' }}; print 'Resuming'
# => Resuming
```

This includes catching built in errors.

```nu
try { 1 / 0 }; print 'Resuming'
# => Resuming
```

The resulting value will be `nothing` if an error occurs and the returned value of the block if an error did not occur.

If you include a `catch` block after the [`try`](/docs/shell/commands/try/) block, it will execute the code in the `catch` block if an error occurred in the [`try`](/docs/shell/commands/try/) block.

```nu
try { 1 / 0 } catch { 'An error happened!' } | $in ++ ' And now I am resuming.'
# => An error happened! And now I am resuming.
```

It will not execute the `catch` block if an error did not occur.

`try` also works for external commands:

```nu
try { ^nonexisting }; print 'a'
# => a

^nonexisting; print 'a'
# => Error: nu::shell::external_command
# => 
# =>   × External command failed
# =>    ╭─[entry #3:1:2]
# =>  1 │ ^nonexisting; print 'a'
# =>    ·  ─────┬─────
# =>    ·       ╰── Command `nonexisting` not found
# =>    ╰────
# =>   help: `nonexisting` is neither a Nushell built-in or a known external command
```

## Other

### `return`

[`return`](/docs/shell/commands/return/) Ends a closure or command early where it is called, without running the rest of the command/closure, and returns the given value. Not often necessary since the last value in a closure or command is also returned, but it can sometimes be convenient.

```nu
def 'positive-check' [it] {
    if $it > 0 {
        return 'positive'
    };

    'non-positive'
}
```

```nu
positive-check 3
# => positive

positive-check (-3)
# => non-positive

let positive_check = {|elt| if $elt > 0 { return 'positive' }; 'non-positive' }

do $positive_check 3
# => positive

do $positive_check (-3)
# => non-positive
```

Ported from [nushell/nushell.github.io](https://github.com/nushell/nushell.github.io). Copyright (c) 2021 Nushell Project. MIT License.
