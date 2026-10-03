---
title: "Expression"
description: "Expression"
order: 1695
section: "Commands · expression"
---




- [`polars arg-where`](/docs/shell/commands/polars_arg-where/) — Creates an expression that returns the arguments where expression is true.
- [`polars as`](/docs/shell/commands/polars_as/) — Creates an alias expression.
- [`polars col`](/docs/shell/commands/polars_col/) — Creates a named column expression.
- [`polars concat-str`](/docs/shell/commands/polars_concat-str/) — Creates a concat string expression.
- [`polars datepart`](/docs/shell/commands/polars_datepart/) — Creates an expression for capturing the specified datepart in a column.
- [`polars horizontal`](/docs/shell/commands/polars_horizontal/) — Horizontal calculation across multiple columns.
- [`polars is-in`](/docs/shell/commands/polars_is-in/) — Creates an is-in expression or checks to see if the elements are contained in the right series
- [`polars lit`](/docs/shell/commands/polars_lit/) — Creates a literal expression.
- [`polars otherwise`](/docs/shell/commands/polars_otherwise/) — Completes a when expression.
- [`polars replace`](/docs/shell/commands/polars_replace/) — Create an expression that replaces old values with new values
- [`polars selector`](/docs/shell/commands/polars_selector/) — Create column selectors for use in polars commands.
- [`polars selector all`](/docs/shell/commands/polars_selector_all/) — Creates a selector that selects all columns.
- [`polars selector alpha`](/docs/shell/commands/polars_selector_alpha/) — Select all columns with alphabetic names (eg: only letters). Matching column names cannot contain *any* non-alphabetic characters. Note that the definition of "alphabetic" consi...
- [`polars selector alphanumeric`](/docs/shell/commands/polars_selector_alphanumeric/) — Select all columns with alphanumeric names (eg: only letters). Matching column names cannot contain *any* non-alphanumeric characters. Note that the definition of "alphanumeric"...
- [`polars selector array`](/docs/shell/commands/polars_selector_array/) — Select all array columns. Optionally filter by fixed width.
- [`polars selector binary`](/docs/shell/commands/polars_selector_binary/) — Select all binary columns.
- [`polars selector boolean`](/docs/shell/commands/polars_selector_boolean/) — Select all boolean columns.
- [`polars selector by-dtype`](/docs/shell/commands/polars_selector_by-dtype/) — Creates a selector that selects columns by data type.
- [`polars selector by-index`](/docs/shell/commands/polars_selector_by-index/) — Select columns by their index position. Supports negative indices (e.g., -1 for the last column).
- [`polars selector by-name`](/docs/shell/commands/polars_selector_by-name/) — Creates a selector that selects columns by name.
- [`polars selector categorical`](/docs/shell/commands/polars_selector_categorical/) — Select all categorical columns.
- [`polars selector contains`](/docs/shell/commands/polars_selector_contains/) — Select columns whose names contain the given literal substring(s).
- [`polars selector date`](/docs/shell/commands/polars_selector_date/) — Select all date columns.
- [`polars selector datetime`](/docs/shell/commands/polars_selector_datetime/) — Select all datetime columns. Optionally filter by time unit (ns, us, ms) and/or timezone.
- [`polars selector decimal`](/docs/shell/commands/polars_selector_decimal/) — Select all decimal columns.
- [`polars selector digit`](/docs/shell/commands/polars_selector_digit/) — Select columns whose names consist entirely of digit characters. By default uses Unicode decimal digits; use `--ascii-only` to restrict to ASCII 0-9.
- [`polars selector duration`](/docs/shell/commands/polars_selector_duration/) — Select all duration columns. Optionally filter by time unit (ns, us, ms).
- [`polars selector empty`](/docs/shell/commands/polars_selector_empty/) — Create an empty selector that matches no columns. Useful as a base for selector composition.
- [`polars selector ends-with`](/docs/shell/commands/polars_selector_ends-with/) — Select columns that end with the given substring(s).
- [`polars selector enum`](/docs/shell/commands/polars_selector_enum/) — Select all enum columns.
- [`polars selector exclude`](/docs/shell/commands/polars_selector_exclude/) — Select all columns except those with the given name(s). This is the inverse of `polars selector by-name`.
- [`polars selector first`](/docs/shell/commands/polars_selector_first/) — Creates a selector that selects the first column(s) by index.
- [`polars selector float`](/docs/shell/commands/polars_selector_float/) — Select all float columns.
- [`polars selector integer`](/docs/shell/commands/polars_selector_integer/) — Select all integer columns.
- [`polars selector last`](/docs/shell/commands/polars_selector_last/) — Creates a selector that selects the last column(s) by index.
- [`polars selector list`](/docs/shell/commands/polars_selector_list/) — Select all list columns.
- [`polars selector matches`](/docs/shell/commands/polars_selector_matches/) — Select all columns that match the given regex pattern.
- [`polars selector nested`](/docs/shell/commands/polars_selector_nested/) — Select all nested columns (list, array, or struct).
- [`polars selector not`](/docs/shell/commands/polars_selector_not/) — Inverts selector.
- [`polars selector numeric`](/docs/shell/commands/polars_selector_numeric/) — Select all numeric columns.
- [`polars selector object`](/docs/shell/commands/polars_selector_object/) — Select all object columns.
- [`polars selector signed-integer`](/docs/shell/commands/polars_selector_signed-integer/) — Select all signed integer columns.
- [`polars selector starts-with`](/docs/shell/commands/polars_selector_starts-with/) — Select columns that start with the given substring(s).
- [`polars selector string`](/docs/shell/commands/polars_selector_string/) — Select all string columns. Use `--include-categorical` to also select categorical columns.
- [`polars selector struct`](/docs/shell/commands/polars_selector_struct/) — Select all struct columns.
- [`polars selector temporal`](/docs/shell/commands/polars_selector_temporal/) — Select all temporal columns (date, datetime, duration, and time).
- [`polars selector unsigned-integer`](/docs/shell/commands/polars_selector_unsigned-integer/) — Select all unsigned integer columns.
- [`polars truncate`](/docs/shell/commands/polars_truncate/) — Divide the date/datetime range into buckets.
- [`polars when`](/docs/shell/commands/polars_when/) — Creates and modifies a when expression.

Ported from [nushell/nushell.github.io](https://github.com/nushell/nushell.github.io). Copyright (c) 2021 Nushell Project. MIT License.
