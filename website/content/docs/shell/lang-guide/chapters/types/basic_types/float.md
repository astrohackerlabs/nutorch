---
title: "Float"
description: "Float"
order: 1099
section: "Language"
---

|                       |                                                                                  |
| --------------------- | -------------------------------------------------------------------------------- |
| **_Description:_**    | Numbers with some fractional component                                           |
| **_Annotation:_**     | `float`                                                                          |
| **_Literal syntax:_** | A decimal numeric value including a decimal place. E.g., `1.5`, `2.0`, `-15.333` |
| **_Casts:_**          | [`into float`](/docs/shell/commands/into_float/)                                     |
| **_See also:_**       | [Types of Data - Floats](/docs/shell/book/types_of_data/#floatsdecimals)                  |

## Additional language notes

- Floats are internally represented as IEEE-754 floats with 64 bit precision.

<!-- TBD: semantics for comparison, NaN/InF. Future hashing. -->

Ported from [nushell/nushell.github.io](https://github.com/nushell/nushell.github.io). Copyright (c) 2021 Nushell Project. MIT License.
