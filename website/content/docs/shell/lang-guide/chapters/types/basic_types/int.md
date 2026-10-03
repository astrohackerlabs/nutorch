---
title: "Integer"
description: "Integer"
order: 1101
section: "Language"
---

|                     |                                                                                                                                                           |
| ------------------- | --------------------------------------------------------------------------------------------------------------------------------------------------------- |
| **Description:**    | Numbers without a fractional component (positive, negative, and 0)                                                                                        |
| **Annotation:**     | `int`                                                                                                                                                     |
| **Literal syntax:** | A decimal, hex, octal, or binary numeric value without a decimal place. E.g., `-100`, `0`, `50`, `+50`, `0xff` (hex), `0o234` (octal), `0b10101` (binary) |
| **Casts**:          | [`into int`](/docs/shell/commands/into_int/)                                                                                                                  |
| **See also:**       | [Types of Data - Integers](/docs/shell/book/types_of_data/#integers)                                                                                               |

## Additional Language Notes

- Integers are internally represented as signed 64-bit numbers with two's-complement arithmetic.

Ported from [nushell/nushell.github.io](https://github.com/nushell/nushell.github.io). Copyright (c) 2021 Nushell Project. MIT License.
