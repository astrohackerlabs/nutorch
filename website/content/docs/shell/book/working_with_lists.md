---
title: "Working with Lists"
description: "Working with Lists"
order: 1015
section: "Book · Nu Fundamentals"
---

> **tip**
> Lists are equivalent to the individual columns of tables. You can think of a list as essentially being a "one-column table" (with no column name). Thus, any command which operates on a column _also_ operates on a list. For instance, [`where`](/docs/shell/commands/where/) can be used with lists:
> 
> ```nu
> [bell book candle] | where ($it =~ 'b')
> # => ╭───┬──────╮
> # => │ 0 │ bell │
> # => │ 1 │ book │
> # => ╰───┴──────╯
> ```
> 


## Creating lists

A list is an ordered collection of values.
A list is created using square brackets surrounding values separated by spaces, linebreaks, and/or commas.
For example, `[foo bar baz]` or `[foo, bar, baz]`.

> **tip**
> Nushell lists are similar to JSON arrays. The same `[ "Item1", "Item2", "Item3" ]` that represents a JSON array can also be used to create a Nushell list.


## Updating lists

We can [`insert`](/docs/shell/commands/insert/) values into lists as they flow through the pipeline, for example let's insert the value `10` into the middle of a list:

```nu
[1, 2, 3, 4] | insert 2 10
# => [1, 2, 10, 3, 4]
```

We can also use [`update`](/docs/shell/commands/update/) to replace the 2nd element with the value `10`.

```nu
[1, 2, 3, 4] | update 1 10
# => [1, 10, 3, 4]
```

## Removing or Adding Items from List

In addition to [`insert`](/docs/shell/commands/insert/) and [`update`](/docs/shell/commands/update/), we also have [`prepend`](/docs/shell/commands/prepend/) and [`append`](/docs/shell/commands/append/). These let you insert to the beginning of a list or at the end of the list, respectively.

For example:

```nu
let colors = [yellow green]
let colors = ($colors | prepend red)
let colors = ($colors | append purple)
let colors = ($colors ++ ["blue"])
let colors = (["black"] ++ $colors)
$colors
# => [black red yellow green purple blue]
```

In case you want to remove items from list, there are many ways. [`skip`](/docs/shell/commands/skip/) allows you skip first rows from input, while [`drop`](/docs/shell/commands/drop/) allows you to skip specific numbered rows from end of list.

```nu
let colors = [red yellow green purple]
let colors = ($colors | skip 1)
let colors = ($colors | drop 2)
$colors
# => [yellow]
```

We also have [`last`](/docs/shell/commands/last/) and [`first`](/docs/shell/commands/first/) which allow you to [`take`](/docs/shell/commands/take/) from the end or beginning of the list, respectively.

```nu
let colors = [red yellow green purple black magenta]
let colors = ($colors | last 3)
$colors
# => [purple black magenta]
```

And from the beginning of a list,

```nu
let colors = [yellow green purple]
let colors = ($colors | first 2)
$colors
# => [yellow green]
```

### Using the Spread Operator

To append one or more lists together, optionally with values interspersed in between, you can also use the
[spread operator](/docs/shell/book/operators/#spread-operator) (`...`):

```nu
let x = [1 2]
[
  ...$x
  3
  ...(4..7 | take 2)
]
# => ╭───┬───╮
# => │ 0 │ 1 │
# => │ 1 │ 2 │
# => │ 2 │ 3 │
# => │ 3 │ 4 │
# => │ 4 │ 5 │
# => ╰───┴───╯
```

## Iterating over Lists

To iterate over the items in a list, use the [`each`](/docs/shell/commands/each/) command with a [block](/docs/shell/book/types_of_data/#blocks)
of Nu code that specifies what to do to each item. The block parameter (e.g. `|elt|` in `{ |elt| print $elt }`) is the current list
item, but the [`enumerate`](/docs/shell/commands/enumerate/) filter can be used to provide `index` and `item` values if needed. For example:

```nu
let names = [Mark Tami Amanda Jeremy]
$names | each { |elt| $"Hello, ($elt)!" }
# Outputs "Hello, Mark!" and three more similar lines.

$names | enumerate | each { |elt| $"($elt.index + 1) - ($elt.item)" }
# Outputs "1 - Mark", "2 - Tami", etc.
```

The [`where`](/docs/shell/commands/where/) command can be used to create a subset of a list, effectively filtering the list based on a condition.

The following example gets all the colors whose names end in "e".

```nu
let colors = [red orange yellow green blue purple]
$colors | where ($it | str ends-with 'e')
# The block passed to `where` must evaluate to a boolean.
# This outputs the list [orange blue purple].
```

In this example, we keep only values higher than `7`.

```nu
let scores = [7 10 8 6 7]
$scores | where $it > 7 # [10 8]
```

The [`reduce`](/docs/shell/commands/reduce/) command computes a single value from a list.
It uses a block which takes 2 parameters: the current item (conventionally named `elt`) and an accumulator
(conventionally named `acc`). To specify an initial value for the accumulator, use the `--fold` (`-f`) flag.
To change `elt` to have `index` and `item` values, use the [`enumerate`](/docs/shell/commands/enumerate/) filter.
For example:

```nu
let scores = [3 8 4]
$"total = ($scores | reduce { |elt, acc| $acc + $elt })" # total = 15

$"total = ($scores | math sum)" # easier approach, same result

$"product = ($scores | reduce --fold 1 { |elt, acc| $acc * $elt })" # product = 96

$scores | enumerate | reduce --fold 0 { |elt, acc| $acc + $elt.index * $elt.item } # 0*3 + 1*8 + 2*4 = 16
```

## Accessing the List

> **tip Note**
> The following is a basic overview. For a more in-depth discussion of this topic, see the chapter, [Navigating and Accessing Structured Data](/docs/shell/book/navigating_structured_data/).


To access a list item at a given index, use the `$name.index` form where `$name` is a variable that holds a list.

For example, the second element in the list below can be accessed with `$names.1`.

```nu
let names = [Mark Tami Amanda Jeremy]
$names.1 # gives Tami
```

If the index is in some variable `$index` we can use the `get` command to extract the item from the list.

```nu
let names = [Mark Tami Amanda Jeremy]
let index = 1
$names | get $index # gives Tami
```

The [`length`](/docs/shell/commands/length/) command returns the number of items in a list.
For example, `[red green blue] | length` outputs `3`.

The [`is-empty`](/docs/shell/commands/is-empty/) command determines whether a string, list, or table is empty.
It can be used with lists as follows:

```nu
let colors = [red green blue]
$colors | is-empty # false

let colors = []
$colors | is-empty # true
```

The `in` and `not-in` operators are used to test whether a value is in a list. For example:

```nu
let colors = [red green blue]
'blue' in $colors # true
'yellow' in $colors # false
'gold' not-in $colors # true
```

The [`any`](/docs/shell/commands/any/) command determines if any item in a list
matches a given condition.
For example:

```nu
let colors = [red green blue]
# Do any color names end with "e"?
$colors | any {|elt| $elt | str ends-with "e" } # true

# Is the length of any color name less than 3?
$colors | any {|elt| ($elt | str length) < 3 } # false

let scores = [3 8 4]
# Are any scores greater than 7?
$scores | any {|elt| $elt > 7 } # true

# Are any scores odd?
$scores | any {|elt| $elt mod 2 == 1 } # true
```

The [`all`](/docs/shell/commands/all/) command determines if every item in a list
matches a given condition.
For example:

```nu
let colors = [red green blue]
# Do all color names end with "e"?
$colors | all {|elt| $elt | str ends-with "e" } # false

# Is the length of all color names greater than or equal to 3?
$colors | all {|elt| ($elt | str length) >= 3 } # true

let scores = [3 8 4]
# Are all scores greater than 7?
$scores | all {|elt| $elt > 7 } # false

# Are all scores even?
$scores | all {|elt| $elt mod 2 == 0 } # false
```

## Converting the List

The [`flatten`](/docs/shell/commands/flatten/) command creates a new list from an existing list
by adding items in nested lists to the top-level list.
This can be called multiple times to flatten lists nested at any depth.
For example:

```nu
[1 [2 3] 4 [5 6]] | flatten # [1 2 3 4 5 6]

[[1 2] [3 [4 5 [6 7 8]]]] | flatten | flatten | flatten # [1 2 3 4 5 6 7 8]
```

The [`wrap`](/docs/shell/commands/wrap/) command converts a list to a table. Each list value will
be converted to a separate row with a single column:

```nu
let zones = [UTC CET Europe/Moscow Asia/Yekaterinburg]

# Show world clock for selected time zones
$zones | wrap 'Zone' | upsert Time {|row| (date now | date to-timezone $row.Zone | format date '%Y.%m.%d %H:%M')}
```

Ported from [nushell/nushell.github.io](https://github.com/nushell/nushell.github.io). Copyright (c) 2021 Nushell Project. MIT License.
