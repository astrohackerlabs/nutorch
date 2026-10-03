---
title: "Datetime"
description: "Datetime"
order: 1096
section: "Language"
---

<!-- prettier-ignore -->
|     |     |
| --- | --- |
| **_Description:_**    | Represents a specific point in time using international standard date time descriptors 
| **_Annotation:_**     | `datetime`                                                                                 
| **_Literal syntax:_** | RFC 3339                                                                               
|                       | Date-only: `2022-02-02`                                                                
|                       | Date and time (GMT): `2022-02-02T14:30:00`                                             
|                       | Date and time including timezone offset: `2022-02-02T14:30:00+05:00`                   
| **_Casts:_**          | [`into datetime`](/docs/shell/commands/into_datetime/)                                     
| **_See also:_**       | [Types of Data - Dates](/docs/shell/book/types_of_data/#dates)

## Additional language notes

- Dates and times are held together in the `datetime` type. Date values used by the system are timezone-aware. By default, dates use the UTC timezone.

## Common commands that can be used with `datetime`

Many of Nushell's builtin commands are datetime aware and output or use `datetime` values
for fields and expressions. For example:

- `date` and its subcommands
- `format date`
- `ls`
- `ps`
- `sys`

Ported from [nushell/nushell.github.io](https://github.com/nushell/nushell.github.io). Copyright (c) 2021 Nushell Project. MIT License.
