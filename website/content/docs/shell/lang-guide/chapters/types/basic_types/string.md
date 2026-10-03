---
title: "String"
description: "String"
order: 1106
section: "Language"
---

|                       |                                                              |
| --------------------- | ------------------------------------------------------------ |
| **_Description:_**    | A series of characters that represents text                  |
| **_Annotation:_**     | `string`                                                     |
| **_Literal Syntax:_** | See [Working with strings](/docs/shell/book/working_with_strings/)    |
| **_Casts:_**          | [`into string`](/docs/shell/commands/into_string/)               |
| **_See also:_**       | [Handling Strings](/docs/shell/book/loading_data/#handling-strings)   |
|                       | [Types of Data - String](/docs/shell/book/types_of_data/#textstrings) |

## Language Notes:

- Nu supports Unicode strings as the basic text type.
- Internally strings are UTF-8 encoded, to ensure a consistent behavior of string operations across different platforms and simplify interoperability with most platforms and the web.
- They have an associated length and do not rely on the C-style null character for termination.
- As strings have to be valid UTF-8 for effective string operations, they can not be used to represent arbitrary binary data. For this please use the [binary data type](/docs/shell/lang-guide/chapters/types/basic_types/string/).

- Different display operations might impose limitations on which non-printable or printable characters get shown. One relevant area are the ANSI escape sequences that can be used to affect the display on the terminal. Certain operations may choose to ignore those.

- TBD: On which level string indexing should be performed: bytes or Unicode scalars.

## Common commands that work with `string`

Many commands takes strings as inputs or parameters.
These commands work with strings explicitly

- `str (subcommand)`
  - For a complete list of subcommands, see: `help str`
- `into string`
- `ansi strip`
- `is-empty`
- `is-not-empty`

In addition to the above commands, most other `into <type>` commands take strings
as inputs.

## Common operators that work with `string`

- `+` : Concatenate two strings
- `+=` : Mutates a string variable by concatenating its right side value.
- `==` : ' True if 2 strings are equal
- `!=` : True if two strings are not equal
- `>` : True if the left string is greater than the right string
- `>=` : True if the left string is greater or equal than the right string
- `<` : True if the left string is less than the right string
- `<=` : True if the left string is less or equal than the right string

Ported from [nushell/nushell.github.io](https://github.com/nushell/nushell.github.io). Copyright (c) 2021 Nushell Project. MIT License.
