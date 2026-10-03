---
title: "Closure"
description: "Closure"
order: 1095
section: "Language"
---

|                       |                                                                                                                                                 |
| --------------------- | ----------------------------------------------------------------------------------------------------------------------------------------------- |
| **_Description:_**    | An anonymous function, often called a lambda function, which accepts parameters and _closes over_ (i.e., uses) variables from outside its scope |
| **_Annotation:_**     | `closure`                                                                                                                                       |
| **_Literal Syntax:_** | `{\|args\| expressions }` where `\|args\|` is optional.                                                                                         |
| **_Casts:_**          | N/A                                                                                                                                             |
| **_See also:_**       | [Types of Data - Closures](/docs/shell/book/types_of_data/#closures)                                                                                     |

Closures are used in Nu extensively as parameters to iteration style commands like `each`, `filter`, and `reduce`, to name but a few. A closure acts like a custom command that can be invoked either explicitly or by other commands. Closures can take parameters, return values and be passed to commands, either builtin or custom.

## Language Notes

1. A closure can be directly invoked using the [`do`](/docs/shell/commands/do/) command.

   ```nu
   do {|a,b| $a + $b } 34 8
   # => 42
   ```

1. The `|args|` list can also contain 0 arguments (`||`) or more than one argument `|arg1,arg2|`

1. When there are 0 arguments, the `||` is optional as long as the closure cannot be mistaken for a record (which also uses the curly-brace style).

   - An empty (no-op) closure can be represented as `{||}`

1. As in other languages with closures, an important feature is their ability to "close over" variables from the parent scope and use their values even after the parent's scope has ended.

   In the following example:

   - `create_greeter` is a custom command that returns a closure
   - It accepts a `$greeting` argument
   - When the `create_greeter` command's block ends, the `$greeting` variable is out of scope
   - However, the closure that is returned has "captured" the value represented by `$greeting` and can still access it later when the closure itself is called.

   ```nu
   def create_greeter [ greeting: string ]: nothing -> closure {
     {|name| $"($greeting), ($name)" }
   }

   let greet = create_greeter "Hello"
   # Invoke the closure with `do`
   do $greet Dalija
   # => Hello, Dalija
   do $greet Ryan
   # => Hello, Ryan

   # Redefine greet with a new greeting
   let greet = create_greeter "Aloha"
   do $greet Kai
   # => Aloha, Kai
   ```

   Note that the `create_greeter` only needs to be defined once.

1. There are some restrictions on the kind of external values that can be closed over. Only immutable variables like those created with the `let` keyword or parameters to a custom command can be captured in a closure. Mutable variables created with the `mut` keyword cannot be captured in a closure. However, you can mutate an `$env` variable if used by the `--env` flag passed to the `do` keyword.

   If we try to create a closure that attempts to capture a mutable variable we get a compile error:

   ```nu
   if true {
     mut x = 9
     do {|p| $p + $x }
   }
   # => Error: Capture of mutable variable.
   ```

1. You cannot pass a closure to an external command; they are reserved only for Nu usage.

1. As with other types, you can also assign a closure to a variable, and closures can be included as values in a list or record.

   ```nu
   let c = {|x| $x + 1 }
   do $c 1
   # => 2
   ```

   ```nu
   let c = [ {|x| $x + 1 } {|x| $x + 2 } ]
   do $c.1 1
   # => 3
   ```

1. You can also use [pipeline input as `$in`](https://www.nushell.sh/lang-guide/chapters/types/basic_types/pipelines.html#pipeline-input-and-the-special-in-variable) in most closures instead of providing an explicit parameter. For example:

   ```nu
   1..5 | each { print $in }
   # => 1
   # => 2
   # => 3
   # => 4
   # => 5
   # => ╭────────────╮
   # => │ empty list │
   # => ╰────────────╯
   ```

1. You can also pass closures themselves into a pipeline assuming the next command knows how to consume it. For example, the `do` example can be rewritten as:

   ```nu
   {|a,b| $a + $b} | do $in 34 8
   # => 43
   ```

1. As seen above, closures can be returned from a custom command. They can also be returned from another closure.

   ```nu
   do {|| {|| 3 }} | do $in
   # => 3
   ```

1. As closures are closely related to functions or commands, their parameters can be typed.

   ```nu
   do {|a:int,b:int| $a + $b } 34 8
   # => 42
   ```

## Common commands that can be used with a `closure`

- [`all`](/docs/shell/commands/all/)
- [`any`](/docs/shell/commands/any/)
- [`collect`](/docs/shell/commands/collect/)
- [`do`](/docs/shell/commands/do/)
- [`each`](/docs/shell/commands/each/)
- [`explain`](/docs/shell/commands/explain/)
- [`filter`](/docs/shell/commands/filter/)
- [`group-by`](/docs/shell/commands/group-by/)
- [`interleave`](/docs/shell/commands/interleave/)
- [`items`](/docs/shell/commands/items/)
- [`par-each`](/docs/shell/commands/par-each/)
- [`reduce`](/docs/shell/commands/reduce/)
- [`skip until`](/docs/shell/commands/skip_until/)
- [`skip while`](/docs/shell/commands/skip_while/)
- [`take until`](/docs/shell/commands/take_until/)
- [`tee`](/docs/shell/commands/tee/)
- [`update`](/docs/shell/commands/update/)
- [`upsert`](/docs/shell/commands/upsert/)
- [`zip`](/docs/shell/commands/zip/)

### Examples of using closures

Here are a few select, concise examples to illustrate the broad use of closures with some of the aforementioned common Nushell commands:

The `each` command iterates over input, applying a closure to transform each item.

```nu
[1 2 3] | each {|num| $num * 10 }
# => ╭───┬────╮
# => │ 0 │ 10 │
# => │ 1 │ 20 │
# => │ 2 │ 30 │
# => ╰───┴────╯
```

The `where` command filters data based on the result of a closure (true: keep, false: reject).

```nu
1..100 | where {|num| $num > 80 and $num < 86 }
# => ╭───┬────╮
# => │ 0 │ 81 │
# => │ 1 │ 82 │
# => │ 2 │ 83 │
# => │ 3 │ 84 │
# => │ 4 │ 85 │
# => ╰───┴────╯
```

The `sort-by` command sorts a list or table. The closure determines and returns the sort key.

```nu
["apple" "banana" "kiwi"] | sort-by {|fruit_name| $fruit_name | str length }
# => ╭───┬────────╮
# => │ 0 │ kiwi   │
# => │ 1 │ apple  │
# => │ 2 │ banana │
# => ╰───┴────────╯
```

The `reduce` command processes a list to accumulate a single result. The closure defines how to combine the current item with the (intermediate) accumulated value.

```nu
[1 2 3 4] | reduce {|accumulator, current_value| $accumulator + $current_value }
# => 10
```

Ported from [nushell/nushell.github.io](https://github.com/nushell/nushell.github.io). Copyright (c) 2021 Nushell Project. MIT License.
