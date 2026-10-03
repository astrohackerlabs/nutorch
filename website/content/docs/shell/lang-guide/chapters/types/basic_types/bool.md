---
title: "Boolean"
description: "Boolean"
order: 1093
section: "Language"
---

|                       |                                                             |
| --------------------- | ----------------------------------------------------------- |
| **_Description:_**    | A logical data type that can have only True or False values |
| **_Annotation:_**     | `bool`                                                      |
| **_Literal syntax:_** | Either a literal `true` or `false`                          |
| **_Casts:_**          | [`into bool`](/docs/shell/commands/into_bool/)                  |
| **_See also:_**       | [Types of Data - Booleans](/docs/shell/book/types_of_data/#booleans) |

## Common commands that can be used with `bool`

- `if`, `while`
- `match` (in clauses where the expression matches the clause expression, or the `_` value which is always true)

- `is-empty`, `is-not-empty`
- `is-admin`
- `is-terminal`

The following commands take a closure as their main argument. The return value from the closure must be a boolean:

- `where`/`filter`
- `any`, `all`, `skip until`, `skip while`, `take until`, `take while`

## Common operators that can be used with `bool`

- `==`, `!=`, `<`, `<=`, `>`, `>=`
- `and`, `or`, `not`
- `in`
- Regex comparison operators: `=~`, `!~` `<regex>`
- String comparison operators: `ends-with`, `starts-with`

Ported from [nushell/nushell.github.io](https://github.com/nushell/nushell.github.io). Copyright (c) 2021 Nushell Project. MIT License.
