---
title: "Record"
description: "Record"
order: 1105
section: "Language"
---

|                       |                                                                                                                |
| --------------------- | -------------------------------------------------------------------------------------------------------------- |
| **_Description:_**    | The foundational associative map. Holds key-value pairs, which associate string keys with various data values. |
| **_Annotation:_**     | `record`                                                                                                       |
| **_Literal syntax:_** | See below                                                                                                      |
| **_Casts:_**          | [`into record`](/docs/shell/commands/into_record/), [`wrap`](/docs/shell/commands/wrap/)                               |
| **_See Also:_**       | [Working with Records](/docs/shell/book/working_with_records/)                                                          |
|                       | [Navigating and Accessing Structured Data](/docs/shell/book/navigating_structured_data/)                                |
|                       | [Types of Data - Records](/docs/shell/book/types_of_data/#records)                                                      |

# Language Notes

- The keys maintain the order of insertion or the order defined in a record literal.
- Keys are guaranteed to be unique. Inserting the same key twice will keep only the last insertion or definition.

(TBD: complex hashable/equality checkable keys)

# Record-Literal Syntax

Record syntax is very similar to objects in JSON. However, commas are _not_ required to separate values when Nushell can easily distinguish them. The key-value pairs of a record may be delimited by:

- Commas

  ```nu
  > {name: "Sam", rank: 10}
  ╭──────┬─────╮
  │ name │ Sam │
  │ rank │ 10  │
  ╰──────┴─────╯
  ```

- Spaces (when unambiguous):

  ```nu
  > {name: "Sam" rank: 10}
  ╭──────┬─────╮
  │ name │ Sam │
  │ rank │ 10  │
  ╰──────┴─────╯
  ```

- Line breaks:

  ```nu
  > {
      name: "Sam"
      rank: 10
    }
  ╭──────┬─────╮
  │ name │ Sam │
  │ rank │ 10  │
  ╰──────┴─────╯
  ```

## Common commands that can be used with `record`

Since the record data type is foundational to Nushell's structured nature, many commands use records as inputs or as parameters. See the list of commands for tables as many of those also take records.

Here are a few commands that use records:

- `get`
- `insert`
- `merge`
- `update`
- `upsert`

Ported from [nushell/nushell.github.io](https://github.com/nushell/nushell.github.io). Copyright (c) 2021 Nushell Project. MIT License.
