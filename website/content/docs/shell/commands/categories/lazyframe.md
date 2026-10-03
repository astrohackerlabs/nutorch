---
title: "Lazyframe"
description: "Lazyframe"
order: 1256
section: "Commands · lazyframe"
---




- [`polars agg`](/docs/shell/commands/polars_agg/) — Performs a series of aggregations from a group-by.
- [`polars collect`](/docs/shell/commands/polars_collect/) — Collect lazy dataframe into eager dataframe.
- [`polars explode`](/docs/shell/commands/polars_explode/) — Explodes a dataframe or creates a explode expression.
- [`polars fill-nan`](/docs/shell/commands/polars_fill-nan/) — Replaces NaN values with the given expression.
- [`polars fill-null`](/docs/shell/commands/polars_fill-null/) — Replaces NULL values with the given expression.
- [`polars filter`](/docs/shell/commands/polars_filter/) — Filter dataframe based in expression.
- [`polars flatten`](/docs/shell/commands/polars_flatten/) — An alias for polars explode.
- [`polars group-by`](/docs/shell/commands/polars_group-by/) — Creates a group-by object that can be used for other aggregations.
- [`polars into-lazy`](/docs/shell/commands/polars_into-lazy/) — Converts a dataframe into a lazy dataframe.
- [`polars join`](/docs/shell/commands/polars_join/) — Joins a lazy frame with other lazy frame.
- [`polars join-where`](/docs/shell/commands/polars_join-where/) — Joins a lazy frame with other lazy frame based on conditions.
- [`polars median`](/docs/shell/commands/polars_median/) — Median value from columns in a dataframe or creates expression for an aggregation
- [`polars over`](/docs/shell/commands/polars_over/) — Compute expressions over a window group defined by partition expressions.
- [`polars quantile`](/docs/shell/commands/polars_quantile/) — Aggregates the columns to the selected quantile.
- [`polars save`](/docs/shell/commands/polars_save/) — Saves a dataframe to disk. For lazy dataframes a sink operation will be used if the file type supports it (parquet, ipc/arrow, csv, and ndjson).
- [`polars select`](/docs/shell/commands/polars_select/) — Selects columns from lazyframe.
- [`polars sort-by`](/docs/shell/commands/polars_sort-by/) — Sorts a lazy dataframe based on expression(s).

Ported from [nushell/nushell.github.io](https://github.com/nushell/nushell.github.io). Copyright (c) 2021 Nushell Project. MIT License.
