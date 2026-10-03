---
title: "Command Reference"
description: "Command Reference"
order: 1138
section: "Commands"
---

If you're new to Nushell, [the quick tour](/docs/shell/book/quick_tour/) can show you the most important commands. You don't need to know them all!

To see all commands from inside Nushell, run [`help commands`](/docs/shell/commands/help/).



## bits

- [`bits`](/docs/shell/commands/bits/) — Various commands for working with bits.
- [`bits and`](/docs/shell/commands/bits_and/) — Performs bitwise and for ints or binary values.
- [`bits not`](/docs/shell/commands/bits_not/) — Performs logical negation on each bit.
- [`bits or`](/docs/shell/commands/bits_or/) — Performs bitwise or for ints or binary values.
- [`bits rol`](/docs/shell/commands/bits_rol/) — Bitwise rotate left for ints or binary values.
- [`bits ror`](/docs/shell/commands/bits_ror/) — Bitwise rotate right for ints or binary values.
- [`bits shl`](/docs/shell/commands/bits_shl/) — Bitwise shift left for ints or binary values.
- [`bits shr`](/docs/shell/commands/bits_shr/) — Bitwise shift right for ints or binary values.
- [`bits xor`](/docs/shell/commands/bits_xor/) — Performs bitwise xor for ints or binary values.

## bytes

- [`bytes`](/docs/shell/commands/bytes/) — Various commands for working with byte data.
- [`bytes add`](/docs/shell/commands/bytes_add/) — Add specified bytes to the binary input.
- [`bytes at`](/docs/shell/commands/bytes_at/) — Get bytes from the input defined by a range.
- [`bytes build`](/docs/shell/commands/bytes_build/) — Create a binary value from the provided arguments.
- [`bytes collect`](/docs/shell/commands/bytes_collect/) — Concatenate multiple binary into a single binary, with an optional separator between each.
- [`bytes ends-with`](/docs/shell/commands/bytes_ends-with/) — Check if binary data ends with a pattern.
- [`bytes index-of`](/docs/shell/commands/bytes_index-of/) — Returns start index of first occurrence of pattern in bytes, or -1 if no match.
- [`bytes length`](/docs/shell/commands/bytes_length/) — Output the length of any bytes in the pipeline.
- [`bytes remove`](/docs/shell/commands/bytes_remove/) — Remove specified bytes from the input.
- [`bytes replace`](/docs/shell/commands/bytes_replace/) — Find and replace bytes in binary data.
- [`bytes reverse`](/docs/shell/commands/bytes_reverse/) — Reverse the bytes in the pipeline.
- [`bytes split`](/docs/shell/commands/bytes_split/) — Split input into multiple items using a separator.
- [`bytes starts-with`](/docs/shell/commands/bytes_starts-with/) — Check if binary data starts with a pattern.

## chart

- [`histogram`](/docs/shell/commands/histogram/) — Creates a new table with a histogram based on the column name passed in.

## conversions

- [`fill`](/docs/shell/commands/fill/) — Fill and align text in columns.
- [`format bits`](/docs/shell/commands/format_bits/) — Convert value to a string of binary data represented by 0 and 1.
- [`format number`](/docs/shell/commands/format_number/) — Format a number.
- [`into`](/docs/shell/commands/into/) — Commands to convert data from one type to another.
- [`into binary`](/docs/shell/commands/into_binary/) — Convert value to a binary primitive.
- [`into bool`](/docs/shell/commands/into_bool/) — Convert value to a boolean.
- [`into cell-path`](/docs/shell/commands/into_cell-path/) — Convert value to a cell-path.
- [`into datetime`](/docs/shell/commands/into_datetime/) — Convert text or timestamp into a datetime.
- [`into duration`](/docs/shell/commands/into_duration/) — Convert value to a duration.
- [`into filesize`](/docs/shell/commands/into_filesize/) — Convert value to a filesize.
- [`into float`](/docs/shell/commands/into_float/) — Convert data into floating point number.
- [`into glob`](/docs/shell/commands/into_glob/) — Convert value to a glob pattern.
- [`into int`](/docs/shell/commands/into_int/) — Convert value to an integer.
- [`into matrix`](/docs/shell/commands/into_matrix/) — Convert a nushell table or list of lists into a matrix.
- [`into record`](/docs/shell/commands/into_record/) — Convert value to a record.
- [`into semver`](/docs/shell/commands/into_semver/) — Convert a value (string, record, or semver) to a semantic version.
- [`into semver-range`](/docs/shell/commands/into_semver-range/) — Convert a string to a semver range.
- [`into sqlite`](/docs/shell/commands/into_sqlite/) — Convert table into a SQLite database.
- [`into string`](/docs/shell/commands/into_string/) — Convert value to a string.
- [`into value`](/docs/shell/commands/into_value/) — Convert custom values into base values.
- [`matrix into-nu`](/docs/shell/commands/matrix_into-nu/) — Convert a matrix to a nushell table (list of lists by default).
- [`split cell-path`](/docs/shell/commands/split_cell-path/) — Split a cell-path into its components.

## core

- [`alias`](/docs/shell/commands/alias/) — Alias a command (with optional flags) to a new name.
- [`attr`](/docs/shell/commands/attr/) — Various attributes for custom commands.
- [`attr category`](/docs/shell/commands/attr_category/) — Attribute for adding a category to custom commands.
- [`attr complete`](/docs/shell/commands/attr_complete/) — Attribute for using another command as a completion source for all arguments.
- [`attr complete external`](/docs/shell/commands/attr_complete_external/) — Attribute for enabling use of the external completer for internal commands.
- [`attr deprecated`](/docs/shell/commands/attr_deprecated/) — Attribute for marking a command or flag as deprecated.
- [`attr example`](/docs/shell/commands/attr_example/) — Attribute for adding examples to custom commands.
- [`attr search-terms`](/docs/shell/commands/attr_search-terms/) — Attribute for adding search terms to custom commands.
- [`break`](/docs/shell/commands/break/) — Break a loop.
- [`collect`](/docs/shell/commands/collect/) — Collect a stream into a value.
- [`commandline`](/docs/shell/commands/commandline/) — View the current command line input buffer.
- [`commandline complete`](/docs/shell/commands/commandline_complete/) — Complete a string using the default completions.
- [`commandline edit`](/docs/shell/commands/commandline_edit/) — Modify the current command line input buffer.
- [`commandline get-cursor`](/docs/shell/commands/commandline_get-cursor/) — Get the current cursor position.
- [`commandline set-cursor`](/docs/shell/commands/commandline_set-cursor/) — Set the current cursor position.
- [`commandline set-prompt`](/docs/shell/commands/commandline_set-prompt/) — Replace the current prompt and repaint it in place, without disturbing the line being edited.
- [`const`](/docs/shell/commands/const/) — Create a parse-time constant.
- [`continue`](/docs/shell/commands/continue/) — Continue a loop from the next iteration.
- [`def`](/docs/shell/commands/def/) — Define a custom command.
- [`describe`](/docs/shell/commands/describe/) — Describe the type and structure of the value(s) piped in.
- [`do`](/docs/shell/commands/do/) — Run a closure, providing it with the pipeline input.
- [`echo`](/docs/shell/commands/echo/) — Returns its arguments, ignoring the piped-in value.
- [`error`](/docs/shell/commands/error/) — Various commands for working with errors.
- [`error make`](/docs/shell/commands/error_make/) — Create an error.
- [`export`](/docs/shell/commands/export/) — Export definitions or environment variables from a module.
- [`export alias`](/docs/shell/commands/export_alias/) — Alias a command (with optional flags) to a new name and export it from a module.
- [`export const`](/docs/shell/commands/export_const/) — Use parse-time constant from a module and export them from this module.
- [`export def`](/docs/shell/commands/export_def/) — Define a custom command and export it from a module.
- [`export extern`](/docs/shell/commands/export_extern/) — Define an extern and export it from a module.
- [`export module`](/docs/shell/commands/export_module/) — Export a custom module from a module.
- [`export use`](/docs/shell/commands/export_use/) — Use definitions from a module and export them from this module.
- [`extern`](/docs/shell/commands/extern/) — Define a signature for an external command.
- [`for`](/docs/shell/commands/for/) — Loop over a range.
- [`help`](/docs/shell/commands/help/) — Display help information about different parts of Nushell.
- [`help aliases`](/docs/shell/commands/help_aliases/) — Show help on nushell aliases.
- [`help commands`](/docs/shell/commands/help_commands/) — Show help on nushell commands.
- [`help escapes`](/docs/shell/commands/help_escapes/) — Show help on nushell string escapes.
- [`help externs`](/docs/shell/commands/help_externs/) — Show help on nushell externs.
- [`help modules`](/docs/shell/commands/help_modules/) — Show help on nushell modules.
- [`help operators`](/docs/shell/commands/help_operators/) — Show help on nushell operators.
- [`help pipe-and-redirect`](/docs/shell/commands/help_pipe-and-redirect/) — Show help on nushell pipes and redirects.
- [`hide`](/docs/shell/commands/hide/) — Hide definitions in the current scope.
- [`hide-env`](/docs/shell/commands/hide-env/) — Hide environment variables in the current scope.
- [`if`](/docs/shell/commands/if/) — Conditionally run a block.
- [`ignore`](/docs/shell/commands/ignore/) — Ignore selected output streams from the previous command in the pipeline.
- [`is-admin`](/docs/shell/commands/is-admin/) — Check if nushell is running with administrator or root privileges.
- [`let`](/docs/shell/commands/let/) — Create a variable and give it a value.
- [`loop`](/docs/shell/commands/loop/) — Run a block in a loop.
- [`match`](/docs/shell/commands/match/) — Conditionally run a block on a matched value.
- [`module`](/docs/shell/commands/module/) — Define a custom module.
- [`mut`](/docs/shell/commands/mut/) — Create a mutable variable and give it a value.
- [`overlay`](/docs/shell/commands/overlay/) — Commands for manipulating overlays.
- [`overlay hide`](/docs/shell/commands/overlay_hide/) — Hide an active overlay.
- [`overlay list`](/docs/shell/commands/overlay_list/) — List all overlays with their active status.
- [`overlay new`](/docs/shell/commands/overlay_new/) — Create an empty overlay.
- [`overlay use`](/docs/shell/commands/overlay_use/) — Use definitions from a module as an overlay.
- [`return`](/docs/shell/commands/return/) — Return early from a custom command.
- [`run`](/docs/shell/commands/run/) — Runs a script file in an isolated scope as part of a pipeline.
- [`scope`](/docs/shell/commands/scope/) — Commands for getting info about what is in scope.
- [`scope aliases`](/docs/shell/commands/scope_aliases/) — Output info on the aliases in the current scope.
- [`scope commands`](/docs/shell/commands/scope_commands/) — Output info on the commands in the current scope.
- [`scope engine-stats`](/docs/shell/commands/scope_engine-stats/) — Output stats on the engine in the current state, including interactive last-result size info.
- [`scope externs`](/docs/shell/commands/scope_externs/) — Output info on the known externals in the current scope.
- [`scope modules`](/docs/shell/commands/scope_modules/) — Output info on the modules in the current scope.
- [`scope variables`](/docs/shell/commands/scope_variables/) — Output info on the variables in the current scope.
- [`source`](/docs/shell/commands/source/) — Runs a script file in the current context.
- [`source-env`](/docs/shell/commands/source-env/) — Source the environment from a source file into the current environment.
- [`try`](/docs/shell/commands/try/) — Try to run a block, if it fails optionally run a catch closure.
- [`use`](/docs/shell/commands/use/) — Use definitions from a module, making them available in your shell.
- [`version`](/docs/shell/commands/version/) — Display Nu version, and its build configuration.
- [`while`](/docs/shell/commands/while/) — Conditionally run a block in a loop.

## database

- [`query db`](/docs/shell/commands/query_db/) — Query a SQLite database with SQL statements.
- [`schema`](/docs/shell/commands/schema/) — Show the schema of a SQLite database.
- [`stor`](/docs/shell/commands/stor/) — Various commands for working with the in-memory sqlite database.
- [`stor create`](/docs/shell/commands/stor_create/) — Create a table in the in-memory sqlite database.
- [`stor delete`](/docs/shell/commands/stor_delete/) — Delete a table or specified rows in the in-memory sqlite database.
- [`stor export`](/docs/shell/commands/stor_export/) — Export the in-memory sqlite database to a sqlite database file.
- [`stor import`](/docs/shell/commands/stor_import/) — Import a sqlite database file into the in-memory sqlite database.
- [`stor insert`](/docs/shell/commands/stor_insert/) — Insert information into a specified table in the in-memory sqlite database.
- [`stor open`](/docs/shell/commands/stor_open/) — Opens the in-memory sqlite database.
- [`stor reset`](/docs/shell/commands/stor_reset/) — Reset the in-memory database by dropping all tables.
- [`stor update`](/docs/shell/commands/stor_update/) — Update information in a specified table in the in-memory sqlite database.

## dataframe

- [`polars`](/docs/shell/commands/polars/) — Operate with data in a dataframe format.
- [`polars agg-groups`](/docs/shell/commands/polars_agg-groups/) — Creates an agg_groups expression.
- [`polars all-false`](/docs/shell/commands/polars_all-false/) — Returns true if all values are false.
- [`polars all-true`](/docs/shell/commands/polars_all-true/) — Returns true if all values are true.
- [`polars append`](/docs/shell/commands/polars_append/) — Appends a new dataframe.
- [`polars arg-max`](/docs/shell/commands/polars_arg-max/) — Return index for max value in series.
- [`polars arg-min`](/docs/shell/commands/polars_arg-min/) — Return index for min value in series.
- [`polars arg-sort`](/docs/shell/commands/polars_arg-sort/) — Returns indexes for a sorted series.
- [`polars arg-true`](/docs/shell/commands/polars_arg-true/) — Returns indexes where values are true.
- [`polars arg-unique`](/docs/shell/commands/polars_arg-unique/) — Returns indexes for unique values.
- [`polars as-date`](/docs/shell/commands/polars_as-date/) — Converts string to date.
- [`polars as-datetime`](/docs/shell/commands/polars_as-datetime/) — Converts string to datetime.
- [`polars cache`](/docs/shell/commands/polars_cache/) — Caches operations in a new LazyFrame.
- [`polars cast`](/docs/shell/commands/polars_cast/) — Cast a column to a different dtype.
- [`polars columns`](/docs/shell/commands/polars_columns/) — Show dataframe columns.
- [`polars concat`](/docs/shell/commands/polars_concat/) — Concatenate two or more dataframes.
- [`polars contains`](/docs/shell/commands/polars_contains/) — Checks if a pattern is contained in a string.
- [`polars convert-time-zone`](/docs/shell/commands/polars_convert-time-zone/) — Convert datetime to target timezone.
- [`polars count`](/docs/shell/commands/polars_count/) — Returns the number of non-null values in the column.
- [`polars count-null`](/docs/shell/commands/polars_count-null/) — Counts null values.
- [`polars cumulative`](/docs/shell/commands/polars_cumulative/) — Cumulative calculation for a column or series.
- [`polars cut`](/docs/shell/commands/polars_cut/) — Bin continuous values into discrete categories for a series.
- [`polars decimal`](/docs/shell/commands/polars_decimal/) — Converts a string column into a decimal column
- [`polars drop`](/docs/shell/commands/polars_drop/) — Creates a new dataframe by dropping the selected columns.
- [`polars drop-duplicates`](/docs/shell/commands/polars_drop-duplicates/) — Drops duplicate values in dataframe.
- [`polars drop-nulls`](/docs/shell/commands/polars_drop-nulls/) — Drops null values in dataframe.
- [`polars dummies`](/docs/shell/commands/polars_dummies/) — Creates a new dataframe with dummy variables.
- [`polars entropy`](/docs/shell/commands/polars_entropy/) — Compute the entropy as `-sum(pk * log(pk))` where `pk` are discrete probabilities.
- [`polars expr-not`](/docs/shell/commands/polars_expr-not/) — Creates a not expression.
- [`polars first`](/docs/shell/commands/polars_first/) — Show only the first number of rows or create a first expression
- [`polars get`](/docs/shell/commands/polars_get/) — Creates dataframe with the selected columns.
- [`polars get-day`](/docs/shell/commands/polars_get-day/) — Gets day from date.
- [`polars get-hour`](/docs/shell/commands/polars_get-hour/) — Gets hour from datetime.
- [`polars get-minute`](/docs/shell/commands/polars_get-minute/) — Gets minute from date.
- [`polars get-month`](/docs/shell/commands/polars_get-month/) — Gets month from date.
- [`polars get-nanosecond`](/docs/shell/commands/polars_get-nanosecond/) — Gets nanosecond from date.
- [`polars get-ordinal`](/docs/shell/commands/polars_get-ordinal/) — Gets ordinal from date.
- [`polars get-second`](/docs/shell/commands/polars_get-second/) — Gets second from date.
- [`polars get-week`](/docs/shell/commands/polars_get-week/) — Gets week from date.
- [`polars get-weekday`](/docs/shell/commands/polars_get-weekday/) — Gets weekday from date.
- [`polars get-year`](/docs/shell/commands/polars_get-year/) — Gets year from date.
- [`polars implode`](/docs/shell/commands/polars_implode/) — Aggregates values into a list.
- [`polars integer`](/docs/shell/commands/polars_integer/) — Converts a string column into a integer column
- [`polars into-df`](/docs/shell/commands/polars_into-df/) — Converts a list, table or record into a dataframe.
- [`polars into-dtype`](/docs/shell/commands/polars_into-dtype/) — Convert a string to a specific datatype.
- [`polars into-nu`](/docs/shell/commands/polars_into-nu/) — Converts a dataframe or an expression into nushell value for access and exploration.
- [`polars into-repr`](/docs/shell/commands/polars_into-repr/) — Display a dataframe in its repr format.
- [`polars into-schema`](/docs/shell/commands/polars_into-schema/) — Convert a value to a polars schema object
- [`polars is-duplicated`](/docs/shell/commands/polars_is-duplicated/) — Creates mask indicating duplicated values.
- [`polars is-not-null`](/docs/shell/commands/polars_is-not-null/) — Creates mask where value is not null.
- [`polars is-null`](/docs/shell/commands/polars_is-null/) — Creates mask where value is null.
- [`polars is-unique`](/docs/shell/commands/polars_is-unique/) — Creates mask indicating unique values.
- [`polars last`](/docs/shell/commands/polars_last/) — Creates new dataframe with tail rows or creates a last expression.
- [`polars len`](/docs/shell/commands/polars_len/) — Return the number of rows in the context. This is similar to COUNT(*) in SQL.
- [`polars list-contains`](/docs/shell/commands/polars_list-contains/) — Checks if an element is contained in a list.
- [`polars lowercase`](/docs/shell/commands/polars_lowercase/) — Lowercase the strings in the column.
- [`polars map-batches`](/docs/shell/commands/polars_map-batches/) — Map a custom Nushell closure over one or more dataframe columns.
- [`polars math`](/docs/shell/commands/polars_math/) — Collection of math functions to be applied on column expressions.
- [`polars math abs`](/docs/shell/commands/polars_math_abs/) — Compute the absolute values of a column expression.
- [`polars math bitwise-and`](/docs/shell/commands/polars_math_bitwise-and/) — Perform an aggregation of bitwise ANDs over a column expression.
- [`polars math bitwise-count-ones`](/docs/shell/commands/polars_math_bitwise-count-ones/) — Compute the number of set bits for each element in an integer column expression.
- [`polars math bitwise-count-zeros`](/docs/shell/commands/polars_math_bitwise-count-zeros/) — Compute the number of unset bits for each element in an integer column expression.
- [`polars math bitwise-leading-ones`](/docs/shell/commands/polars_math_bitwise-leading-ones/) — Compute the number of leading set bits for each element in an integer column expression.
- [`polars math bitwise-leading-zeros`](/docs/shell/commands/polars_math_bitwise-leading-zeros/) — Compute the number of leading unset bits for each element in an integer column expression.
- [`polars math bitwise-or`](/docs/shell/commands/polars_math_bitwise-or/) — Perform an aggregation of bitwise ORs over a column expression.
- [`polars math bitwise-trailing-ones`](/docs/shell/commands/polars_math_bitwise-trailing-ones/) — Compute the number of trailing set bits for each element in an integer column expression.
- [`polars math bitwise-trailing-zeros`](/docs/shell/commands/polars_math_bitwise-trailing-zeros/) — Compute the number of trailing unset bits for each element in an integer column expression.
- [`polars math bitwise-xor`](/docs/shell/commands/polars_math_bitwise-xor/) — Perform an aggregation of bitwise XORs over a column expression.
- [`polars math cos`](/docs/shell/commands/polars_math_cos/) — Compute the element-wise cosine of a column expression.
- [`polars math dot`](/docs/shell/commands/polars_math_dot/) — Compute the dot product of two column expressions.
- [`polars math exp`](/docs/shell/commands/polars_math_exp/) — Compute element-wise e raised to the power of a column expression.
- [`polars math log`](/docs/shell/commands/polars_math_log/) — Compute the element-wise logarithm of a column expression.
- [`polars math log1p`](/docs/shell/commands/polars_math_log1p/) — Compute the element-wise natural log of 1 + x for a column expression.
- [`polars math sign`](/docs/shell/commands/polars_math_sign/) — Compute the element-wise sign of a column expression, returning -1, 0, or 1.
- [`polars math sin`](/docs/shell/commands/polars_math_sin/) — Compute the element-wise sine of a column expression.
- [`polars math sqrt`](/docs/shell/commands/polars_math_sqrt/) — Compute the element-wise square root of a column expression.
- [`polars max`](/docs/shell/commands/polars_max/) — Creates a max expression or aggregates columns to their max value.
- [`polars mean`](/docs/shell/commands/polars_mean/) — Creates a mean expression for an aggregation or aggregates columns to their mean value.
- [`polars min`](/docs/shell/commands/polars_min/) — Creates a min expression or aggregates columns to their min value.
- [`polars n-unique`](/docs/shell/commands/polars_n-unique/) — Counts unique values.
- [`polars not`](/docs/shell/commands/polars_not/) — Inverts boolean mask.
- [`polars open`](/docs/shell/commands/polars_open/) — Opens CSV, JSON, NDJSON/JSON lines, arrow, avro, or parquet file to create dataframe. A lazy dataframe will be created by default, if supported.
- [`polars pivot`](/docs/shell/commands/polars_pivot/) — Pivot a DataFrame from long to wide format.
- [`polars profile`](/docs/shell/commands/polars_profile/) — Profile a lazy dataframe.
- [`polars qcut`](/docs/shell/commands/polars_qcut/) — Bin continuous values into discrete categories based on their quantiles for a series.
- [`polars query`](/docs/shell/commands/polars_query/) — Query dataframe using SQL. Note: The dataframe is always named 'df' in your query's from clause.
- [`polars replace-time-zone`](/docs/shell/commands/polars_replace-time-zone/) — Replace the timezone information in a datetime column.
- [`polars reverse`](/docs/shell/commands/polars_reverse/) — Reverses the LazyFrame
- [`polars rolling`](/docs/shell/commands/polars_rolling/) — Rolling calculation for a series or expression, or a rolling group-by for a lazyframe.
- [`polars sample`](/docs/shell/commands/polars_sample/) — Create sample dataframe.
- [`polars schema`](/docs/shell/commands/polars_schema/) — Show schema for a dataframe.
- [`polars set`](/docs/shell/commands/polars_set/) — Sets value where given mask is true.
- [`polars set-with-idx`](/docs/shell/commands/polars_set-with-idx/) — Sets value in the given index.
- [`polars shape`](/docs/shell/commands/polars_shape/) — Shows column and row size for a dataframe.
- [`polars slice`](/docs/shell/commands/polars_slice/) — Creates new dataframe from a slice of rows.
- [`polars std`](/docs/shell/commands/polars_std/) — Creates a std expression for an aggregation of std value from columns in a dataframe.
- [`polars store-get`](/docs/shell/commands/polars_store-get/) — Gets a Dataframe or other object from the plugin cache.
- [`polars store-ls`](/docs/shell/commands/polars_store-ls/) — Lists stored polars objects.
- [`polars store-rm`](/docs/shell/commands/polars_store-rm/) — Removes a stored Dataframe or other object from the plugin cache.
- [`polars str-join`](/docs/shell/commands/polars_str-join/) — Concatenates strings within a column or dataframes
- [`polars str-lengths`](/docs/shell/commands/polars_str-lengths/) — Get lengths of all strings.
- [`polars str-replace`](/docs/shell/commands/polars_str-replace/) — Replace the leftmost (sub)string by a regex pattern.
- [`polars str-replace-all`](/docs/shell/commands/polars_str-replace-all/) — Replace all (sub)strings by a regex pattern.
- [`polars str-slice`](/docs/shell/commands/polars_str-slice/) — Slices the string from the start position until the selected length.
- [`polars str-split`](/docs/shell/commands/polars_str-split/) — Split the string by a substring. The resulting dtype is list<str>.
- [`polars str-strip-chars`](/docs/shell/commands/polars_str-strip-chars/) — Strips specified characters from strings in a column
- [`polars strftime`](/docs/shell/commands/polars_strftime/) — Formats date based on string rule.
- [`polars struct-json-encode`](/docs/shell/commands/polars_struct-json-encode/) — Convert this struct to a string column with json values.
- [`polars sum`](/docs/shell/commands/polars_sum/) — Creates a sum expression for an aggregation or aggregates columns to their sum value.
- [`polars summary`](/docs/shell/commands/polars_summary/) — For a dataframe, produces descriptive statistics (summary statistics) for its numeric columns.
- [`polars take`](/docs/shell/commands/polars_take/) — Creates new dataframe using the given indices.
- [`polars unnest`](/docs/shell/commands/polars_unnest/) — Decompose struct columns into separate columns for each of their fields. The new columns will be inserted into the dataframe at the location of the struct column.
- [`polars unpivot`](/docs/shell/commands/polars_unpivot/) — Unpivot a DataFrame from wide to long format.
- [`polars uppercase`](/docs/shell/commands/polars_uppercase/) — Uppercase the strings in the column.
- [`polars value-counts`](/docs/shell/commands/polars_value-counts/) — Returns a dataframe with the counts for unique values in series.
- [`polars var`](/docs/shell/commands/polars_var/) — Create a var expression for an aggregation.

## dataframe or lazyframe

- [`polars filter-with`](/docs/shell/commands/polars_filter-with/) — Filters dataframe using a mask or expression as reference.
- [`polars rename`](/docs/shell/commands/polars_rename/) — Rename a dataframe column.
- [`polars shift`](/docs/shell/commands/polars_shift/) — Shifts the values by a given period.
- [`polars unique`](/docs/shell/commands/polars_unique/) — Returns unique values from a dataframe.
- [`polars with-column`](/docs/shell/commands/polars_with-column/) — Adds a series to the dataframe.

## date

- [`date`](/docs/shell/commands/date/) — Date-related commands.
- [`date from-human`](/docs/shell/commands/date_from-human/) — Convert a human readable datetime string to a datetime.
- [`date humanize`](/docs/shell/commands/date_humanize/) — Print a 'humanized' format for the date, relative to now.
- [`date list-timezone`](/docs/shell/commands/date_list-timezone/) — List supported time zones.
- [`date now`](/docs/shell/commands/date_now/) — Get the current date.
- [`date to-timezone`](/docs/shell/commands/date_to-timezone/) — Convert a date to a given time zone.

## debug

- [`ast`](/docs/shell/commands/ast/) — Print the abstract syntax tree (ast) for a pipeline.
- [`config flatten`](/docs/shell/commands/config_flatten/) — Show the current configuration in a flattened form.
- [`debug`](/docs/shell/commands/debug/) — Debug print the value(s) piped in.
- [`debug env`](/docs/shell/commands/debug_env/) — Show environment variables as external commands would get it.
- [`debug experimental-options`](/docs/shell/commands/debug_experimental-options/) — Show all experimental options.
- [`debug info`](/docs/shell/commands/debug_info/) — View process memory info.
- [`debug profile`](/docs/shell/commands/debug_profile/) — Profile pipeline elements in a closure.
- [`explain`](/docs/shell/commands/explain/) — Explain closure contents.
- [`inspect`](/docs/shell/commands/inspect/) — Inspect pipeline results while running a pipeline.
- [`metadata`](/docs/shell/commands/metadata/) — Get the metadata for items in the stream.
- [`metadata access`](/docs/shell/commands/metadata_access/) — Access the metadata for the input stream within a closure.
- [`metadata set`](/docs/shell/commands/metadata_set/) — Set the metadata for items in the stream.
- [`panic`](/docs/shell/commands/panic/) — Causes nushell to panic.
- [`timeit`](/docs/shell/commands/timeit/) — Time how long it takes a closure to run.
- [`view`](/docs/shell/commands/view/) — Various commands for viewing debug information.
- [`view blocks`](/docs/shell/commands/view_blocks/) — View the blocks registered in nushell's EngineState memory.
- [`view files`](/docs/shell/commands/view_files/) — View the files registered in nushell's EngineState memory.
- [`view ir`](/docs/shell/commands/view_ir/) — View the compiled IR code for a block of code.
- [`view source`](/docs/shell/commands/view_source/) — View a block, module, or a definition.
- [`view span`](/docs/shell/commands/view_span/) — View the contents of a span.

## default

- [`banner`](/docs/shell/commands/banner/) — Print a banner for Nushell with information about the project
- [`inc`](/docs/shell/commands/inc/) — Increment a value or version. Optionally use the column of a table.
- [`peek`](/docs/shell/commands/peek/) — Peek the first <n> elements of a stream and store them in the metadata.
- [`pwd`](/docs/shell/commands/pwd/) — Return the current working directory
- [`run-internal`](/docs/shell/commands/run-internal/) — Run a built-in command by name. Used internally by `%($cmd)` dynamic dispatch.

## env

- [`config`](/docs/shell/commands/config/) — Edit nushell configuration files.
- [`config env`](/docs/shell/commands/config_env/) — Edit nu environment configurations.
- [`config nu`](/docs/shell/commands/config_nu/) — Edit nu configurations.
- [`config reset`](/docs/shell/commands/config_reset/) — Reset nushell environment configurations to default, and saves old config files in the config location as oldconfig.nu and oldenv.nu.
- [`config use-colors`](/docs/shell/commands/config_use-colors/) — Get the configuration for color output.
- [`export-env`](/docs/shell/commands/export-env/) — Run a block and preserve its environment in a current scope.
- [`with-env`](/docs/shell/commands/with-env/) — Runs a block with an environment variable set.

## experimental

- [`job`](/docs/shell/commands/job/) — Various commands for working with background jobs.
- [`job describe`](/docs/shell/commands/job_describe/) — Add a description to a background job.
- [`job flush`](/docs/shell/commands/job_flush/) — Clear this job's mailbox.
- [`job id`](/docs/shell/commands/job_id/) — Get id of current job.
- [`job kill`](/docs/shell/commands/job_kill/) — Kill a background job.
- [`job list`](/docs/shell/commands/job_list/) — List background jobs.
- [`job recv`](/docs/shell/commands/job_recv/) — Read a message from a job's mailbox.
- [`job send`](/docs/shell/commands/job_send/) — Send a message to the mailbox of a job.
- [`job spawn`](/docs/shell/commands/job_spawn/) — Spawn a background job and retrieve its ID.
- [`job unfreeze`](/docs/shell/commands/job_unfreeze/) — Unfreeze a frozen process job in foreground.
- [`unlet`](/docs/shell/commands/unlet/) — Delete variables from nushell memory, making them unrecoverable.

## expression

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

## filesystem

- [`cd`](/docs/shell/commands/cd/) — Change the current working directory.
- [`cp`](/docs/shell/commands/cp/) — Copy files using uutils/coreutils cp.
- [`du`](/docs/shell/commands/du/) — Find disk usage sizes of specified items.
- [`glob`](/docs/shell/commands/glob/) — Creates a list of files and/or folders based on the glob pattern provided.
- [`idx`](/docs/shell/commands/idx/) — Manage in-memory file index state.
- [`idx dirs`](/docs/shell/commands/idx_dirs/) — List indexed directories, or fuzzy-match directories by query.
- [`idx drop`](/docs/shell/commands/idx_drop/) — Drop the current idx runtime from memory.
- [`idx files`](/docs/shell/commands/idx_files/) — List indexed files, or fuzzy-match files by query.
- [`idx find`](/docs/shell/commands/idx_find/) — Search idx with fuzzy matching across files and directories by default.
- [`idx init`](/docs/shell/commands/idx_init/) — Initialize the in-memory idx index for a path.
- [`idx search`](/docs/shell/commands/idx_search/) — Search indexed file contents.
- [`idx status`](/docs/shell/commands/idx_status/) — Show status information for the global in-memory idx runtime.
- [`idx watch`](/docs/shell/commands/idx_watch/) — Stream filesystem change events from the live idx index.
- [`load-env`](/docs/shell/commands/load-env/) — Loads an environment update from a record.
- [`ls`](/docs/shell/commands/ls/) — List the filenames, sizes, and modification times of items in a directory.
- [`mkdir`](/docs/shell/commands/mkdir/) — Create directories, with intermediary directories if required using uutils/coreutils mkdir.
- [`mktemp`](/docs/shell/commands/mktemp/) — Create temporary files or directories using uutils/coreutils mktemp.
- [`mv`](/docs/shell/commands/mv/) — Move files or directories using uutils/coreutils mv.
- [`open`](/docs/shell/commands/open/) — Load a file into a cell, converting to table if possible (avoid by appending '--raw').
- [`rm`](/docs/shell/commands/rm/) — Remove files and directories.
- [`save`](/docs/shell/commands/save/) — Save a file.
- [`start`](/docs/shell/commands/start/) — Open a folder, file, or website in the default application or viewer.
- [`touch`](/docs/shell/commands/touch/) — Creates one or more files.
- [`watch`](/docs/shell/commands/watch/) — Watch for file changes and execute Nu code when they happen.

## filters

- [`all`](/docs/shell/commands/all/) — Test if every element of the input fulfills a predicate expression.
- [`any`](/docs/shell/commands/any/) — Tests if any element of the input fulfills a predicate expression.
- [`append`](/docs/shell/commands/append/) — Append any number of rows to a table.
- [`chunk-by`](/docs/shell/commands/chunk-by/) — Divides a sequence into sub-sequences based on a closure.
- [`chunks`](/docs/shell/commands/chunks/) — Divide a list, table or binary input into chunks of `chunk_size`. For binary input, `chunk_size` can also be specified as a filesize.
- [`columns`](/docs/shell/commands/columns/) — Given a record or table, produce a list of its columns' names.
- [`combinations`](/docs/shell/commands/combinations/) — Generates all combinations of size k from the input list.
- [`compact`](/docs/shell/commands/compact/) — Creates a table with non-empty rows.
- [`default`](/docs/shell/commands/default/) — Sets a default value if a row's column is missing or null.
- [`difference`](/docs/shell/commands/difference/) — Returns a list of unique elements in the input that are not present in the other list.
- [`drop`](/docs/shell/commands/drop/) — Remove items/rows from the end of the input list/table, or remove bytes from the end of binary data. Counterpart of `skip`. Opposite of `last`. For binary input, `rows` can also...
- [`drop column`](/docs/shell/commands/drop_column/) — Remove N columns at the right-hand end of the input table. To remove columns by name, use `reject`.
- [`drop nth`](/docs/shell/commands/drop_nth/) — Drop the selected rows.
- [`each`](/docs/shell/commands/each/) — Run a closure on each row of the input list, creating a new list with the results.
- [`each while`](/docs/shell/commands/each_while/) — Run a closure on each row of the input list until a null is found, then create a new list with the results.
- [`enumerate`](/docs/shell/commands/enumerate/) — Enumerate the elements in a stream.
- [`every`](/docs/shell/commands/every/) — Show (or skip) every n-th row, starting from the first one.
- [`filter`](/docs/shell/commands/filter/) — Filter values based on a predicate closure.
- [`find`](/docs/shell/commands/find/) — Search for terms in the input data.
- [`first`](/docs/shell/commands/first/) — Return only the first several rows of the input. Counterpart of `last`. Opposite of `skip`. For binary input, rows can also be specified as a filesize.
- [`flatten`](/docs/shell/commands/flatten/) — Flatten a table by extracting nested values.
- [`get`](/docs/shell/commands/get/) — Extract data using a cell path.
- [`group-by`](/docs/shell/commands/group-by/) — Splits a list or table into groups, and returns a record containing those groups.
- [`headers`](/docs/shell/commands/headers/) — Use the first row of the table as column names.
- [`insert`](/docs/shell/commands/insert/) — Insert a new column, using an expression or closure to create each row's values.
- [`interleave`](/docs/shell/commands/interleave/) — Read multiple streams in parallel and combine them into one stream.
- [`intersect`](/docs/shell/commands/intersect/) — Returns a list of unique elements present in both the input and the provided list.
- [`is-empty`](/docs/shell/commands/is-empty/) — Check for empty values.
- [`is-not-empty`](/docs/shell/commands/is-not-empty/) — Check for non-empty values.
- [`items`](/docs/shell/commands/items/) — Given a record, iterate on each pair of column name and associated value.
- [`join`](/docs/shell/commands/join/) — Join two tables.
- [`last`](/docs/shell/commands/last/) — Return only the last several rows of the input. Counterpart of `first`. Opposite of `drop`. For binary input, rows can also be specified as a filesize.
- [`length`](/docs/shell/commands/length/) — Count the number of items in an input list, rows in a table, or bytes in binary data.
- [`lines`](/docs/shell/commands/lines/) — Converts input to lines.
- [`matrix`](/docs/shell/commands/matrix/) — Various commands for working with matrices.
- [`matrix add`](/docs/shell/commands/matrix_add/) — Add a matrix or scalar to a matrix.
- [`matrix get-col`](/docs/shell/commands/matrix_get-col/) — Extract a column from a 2D matrix.
- [`matrix get-row`](/docs/shell/commands/matrix_get-row/) — Extract a row from a matrix.
- [`matrix identity`](/docs/shell/commands/matrix_identity/) — Create an identity matrix of the given size.
- [`matrix map`](/docs/shell/commands/matrix_map/) — Apply a closure to each element of a matrix and return a new matrix.
- [`matrix max`](/docs/shell/commands/matrix_max/) — Find the maximum value in a matrix, or max along an axis.
- [`matrix mean`](/docs/shell/commands/matrix_mean/) — Compute the mean of all elements in a matrix.
- [`matrix multiply`](/docs/shell/commands/matrix_multiply/) — Multiply two matrices using dot product.
- [`matrix reduce`](/docs/shell/commands/matrix_reduce/) — Reduce all elements of a matrix to a single value.
- [`matrix reshape`](/docs/shell/commands/matrix_reshape/) — Change the dimensions of a matrix.
- [`matrix scale`](/docs/shell/commands/matrix_scale/) — Multiply all elements of a matrix by a scalar.
- [`matrix set-col`](/docs/shell/commands/matrix_set-col/) — Replace a column in a 2D matrix.
- [`matrix set-row`](/docs/shell/commands/matrix_set-row/) — Replace a row in a matrix.
- [`matrix subtract`](/docs/shell/commands/matrix_subtract/) — Subtract a matrix or scalar from a matrix.
- [`matrix sum`](/docs/shell/commands/matrix_sum/) — Sum all elements of a matrix, or sum along an axis.
- [`matrix transpose`](/docs/shell/commands/matrix_transpose/) — Transpose a matrix (swap rows and columns). For n-dimensional arrays, reverses all axes.
- [`matrix zeros`](/docs/shell/commands/matrix_zeros/) — Create a matrix filled with zeros.
- [`merge`](/docs/shell/commands/merge/) — Merge the input with a record or table, overwriting values in matching columns.
- [`merge deep`](/docs/shell/commands/merge_deep/) — Merge the input with a record or table, recursively merging values in matching columns.
- [`move`](/docs/shell/commands/move/) — Moves columns relative to other columns or make them the first/last columns. Flags are mutually exclusive.
- [`par-each`](/docs/shell/commands/par-each/) — Run a closure on each row of the input list in parallel, creating a new list with the results.
- [`permutations`](/docs/shell/commands/permutations/) — Generates all permutations of the input list.
- [`prepend`](/docs/shell/commands/prepend/) — Prepend any number of rows to a table.
- [`query`](/docs/shell/commands/query/) — Show all the query commands
- [`query json`](/docs/shell/commands/query_json/) — execute json query on json file (open --raw <file> | query json 'query string')
- [`query xml`](/docs/shell/commands/query_xml/) — Execute XPath 1.0 query on XML input
- [`reduce`](/docs/shell/commands/reduce/) — Aggregate a list (starting from the left) to a single value using an accumulator closure.
- [`reject`](/docs/shell/commands/reject/) — Remove the given columns or rows from the table. Opposite of `select`.
- [`rename`](/docs/shell/commands/rename/) — Creates a new table with columns renamed.
- [`reverse`](/docs/shell/commands/reverse/) — Reverses the input list or table.
- [`roll`](/docs/shell/commands/roll/) — Rolling commands for tables.
- [`roll down`](/docs/shell/commands/roll_down/) — Roll table rows down.
- [`roll left`](/docs/shell/commands/roll_left/) — Roll record or table columns left.
- [`roll right`](/docs/shell/commands/roll_right/) — Roll table columns right.
- [`roll up`](/docs/shell/commands/roll_up/) — Roll table rows up.
- [`rotate`](/docs/shell/commands/rotate/) — Rotates a table or record clockwise (default) or counter-clockwise (use --ccw flag).
- [`select`](/docs/shell/commands/select/) — Select only these columns or rows from the input. Opposite of `reject`.
- [`semver`](/docs/shell/commands/semver/) — Various commands for working with semantic versions.
- [`semver bump`](/docs/shell/commands/semver_bump/) — Bump a semantic version to the next level.
- [`shuffle`](/docs/shell/commands/shuffle/) — Shuffle rows randomly.
- [`skip`](/docs/shell/commands/skip/) — Skip the first several rows of the input. Counterpart of `drop`. Opposite of `first`. For binary input, n can also be specified as a filesize.
- [`skip until`](/docs/shell/commands/skip_until/) — Skip elements of the input until a predicate is true.
- [`skip while`](/docs/shell/commands/skip_while/) — Skip elements of the input while a predicate is true.
- [`slice`](/docs/shell/commands/slice/) — Return only the selected rows.
- [`sort`](/docs/shell/commands/sort/) — Sort the input in increasing order.
- [`sort-by`](/docs/shell/commands/sort-by/) — Sort by the given cell path or closure.
- [`split list`](/docs/shell/commands/split_list/) — Split a list into multiple lists using a separator.
- [`take`](/docs/shell/commands/take/) — Take only the first n elements of a list, or the first n bytes of a binary value. For binary input, n can also be specified as a filesize.
- [`take until`](/docs/shell/commands/take_until/) — Take elements of the input until a predicate is true.
- [`take while`](/docs/shell/commands/take_while/) — Take elements of the input while a predicate is true.
- [`tee`](/docs/shell/commands/tee/) — Copy a stream to another command in parallel.
- [`transpose`](/docs/shell/commands/transpose/) — Transposes the table contents so rows become columns and columns become rows.
- [`union`](/docs/shell/commands/union/) — Returns a list of unique elements from both the input and the provided list.
- [`uniq`](/docs/shell/commands/uniq/) — Return the distinct values in the input.
- [`uniq-by`](/docs/shell/commands/uniq-by/) — Return the distinct values in the input by the given column(s).
- [`update`](/docs/shell/commands/update/) — Update an existing column to have a new value.
- [`update cells`](/docs/shell/commands/update_cells/) — Update the table cells.
- [`upsert`](/docs/shell/commands/upsert/) — Update an existing column to have a new value, or insert a new column.
- [`values`](/docs/shell/commands/values/) — Given a record or table, produce a list of its columns' values.
- [`where`](/docs/shell/commands/where/) — Filter values of an input list based on a condition.
- [`window`](/docs/shell/commands/window/) — Creates a sliding window of `window_size` that slide by n rows/elements across input.
- [`wrap`](/docs/shell/commands/wrap/) — Wrap the value into a column.
- [`zip`](/docs/shell/commands/zip/) — Combine a stream with the input.

## formats

- [`decode base32`](/docs/shell/commands/decode_base32/) — Decode a Base32-encoded value.
- [`decode base32hex`](/docs/shell/commands/decode_base32hex/) — Encode a base32hex value.
- [`decode base64`](/docs/shell/commands/decode_base64/) — Decode a Base64-encoded value.
- [`decode hex`](/docs/shell/commands/decode_hex/) — Decode a hex-encoded value.
- [`encode base32`](/docs/shell/commands/encode_base32/) — Encode a string or binary value using Base32.
- [`encode base32hex`](/docs/shell/commands/encode_base32hex/) — Encode a binary value or a string using base32hex.
- [`encode base64`](/docs/shell/commands/encode_base64/) — Encode a string or binary value using Base64.
- [`encode hex`](/docs/shell/commands/encode_hex/) — Hex encode a binary value or a string.
- [`from`](/docs/shell/commands/from/) — Parse a string or binary data into structured data.
- [`from csv`](/docs/shell/commands/from_csv/) — Parse text as .csv and create table.
- [`from eml`](/docs/shell/commands/from_eml/) — Parse text as .eml and create record.
- [`from ics`](/docs/shell/commands/from_ics/) — Parse text as .ics and create table.
- [`from ini`](/docs/shell/commands/from_ini/) — Parse text as .ini and create table.
- [`from json`](/docs/shell/commands/from_json/) — Convert JSON text into structured data.
- [`from kdl`](/docs/shell/commands/from_kdl/) — Convert KDL text into structured data.
- [`from md`](/docs/shell/commands/from_md/) — Convert markdown text into human-friendly structured rows. Use --verbose for the full AST.
- [`from msgpack`](/docs/shell/commands/from_msgpack/) — Convert MessagePack data into Nu values.
- [`from msgpackz`](/docs/shell/commands/from_msgpackz/) — Convert brotli-compressed MessagePack data into Nu values.
- [`from nuon`](/docs/shell/commands/from_nuon/) — Convert from nuon to structured data.
- [`from ods`](/docs/shell/commands/from_ods/) — Parse OpenDocument Spreadsheet(.ods) data and create table.
- [`from plist`](/docs/shell/commands/from_plist/) — Convert plist to Nushell values
- [`from ssv`](/docs/shell/commands/from_ssv/) — Parse text as space-separated values and create a table. The default minimum number of spaces counted as a separator is 2.
- [`from toml`](/docs/shell/commands/from_toml/) — Parse text as .toml and create record.
- [`from tsv`](/docs/shell/commands/from_tsv/) — Parse text as .tsv and create table.
- [`from url`](/docs/shell/commands/from_url/) — Parse url-encoded string as a record.
- [`from vcf`](/docs/shell/commands/from_vcf/) — Parse text as .vcf and create table.
- [`from xlsx`](/docs/shell/commands/from_xlsx/) — Parse binary Excel(.xlsx) data and create table.
- [`from xml`](/docs/shell/commands/from_xml/) — Parse text as .xml and create record.
- [`from yaml`](/docs/shell/commands/from_yaml/) — Parse text as .yaml/.yml and create table.
- [`from yml`](/docs/shell/commands/from_yml/) — Parse text as .yaml/.yml and create table.
- [`to`](/docs/shell/commands/to/) — Translate structured data to various formats.
- [`to csv`](/docs/shell/commands/to_csv/) — Convert table into .csv text .
- [`to html`](/docs/shell/commands/to_html/) — Convert table into simple HTML.
- [`to json`](/docs/shell/commands/to_json/) — Converts table data into JSON text.
- [`to kdl`](/docs/shell/commands/to_kdl/) — Converts structured data into KDL text.
- [`to md`](/docs/shell/commands/to_md/) — Convert table into simple Markdown.
- [`to msgpack`](/docs/shell/commands/to_msgpack/) — Convert Nu values into MessagePack.
- [`to msgpackz`](/docs/shell/commands/to_msgpackz/) — Convert Nu values into brotli-compressed MessagePack.
- [`to nuon`](/docs/shell/commands/to_nuon/) — Converts table data into Nuon (Nushell Object Notation) text.
- [`to plist`](/docs/shell/commands/to_plist/) — Convert Nu values into plist
- [`to text`](/docs/shell/commands/to_text/) — Convert data into plain text format.
- [`to toml`](/docs/shell/commands/to_toml/) — Convert record into .toml text.
- [`to tsv`](/docs/shell/commands/to_tsv/) — Convert table into .tsv text.
- [`to txt`](/docs/shell/commands/to_txt/) — Convert data into plain text format.
- [`to xml`](/docs/shell/commands/to_xml/) — Convert special record structure into .xml text.
- [`to yaml`](/docs/shell/commands/to_yaml/) — Convert table into .yaml/.yml text.
- [`to yml`](/docs/shell/commands/to_yml/) — Convert table into .yaml/.yml text.

## generators

- [`cal`](/docs/shell/commands/cal/) — Display a calendar.
- [`generate`](/docs/shell/commands/generate/) — Generate a list of values by successively invoking a closure.
- [`seq`](/docs/shell/commands/seq/) — Output sequences of numbers.
- [`seq char`](/docs/shell/commands/seq_char/) — Print a sequence of ASCII characters.
- [`seq date`](/docs/shell/commands/seq_date/) — Print sequences of dates.

## hash

- [`hash`](/docs/shell/commands/hash/) — Apply hash function.
- [`hash md5`](/docs/shell/commands/hash_md5/) — Hash a value using the md5 hash algorithm.
- [`hash sha256`](/docs/shell/commands/hash_sha256/) — Hash a value using the sha256 hash algorithm.

## history

- [`history`](/docs/shell/commands/history/) — Get the command history.
- [`history import`](/docs/shell/commands/history_import/) — Import command line history.
- [`history session`](/docs/shell/commands/history_session/) — Get the command history session.

## lazyframe

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

## math

- [`math`](/docs/shell/commands/math/) — Use mathematical functions as aggregate functions on a list of numbers or tables.
- [`math abs`](/docs/shell/commands/math_abs/) — Returns the absolute value of a number.
- [`math arccos`](/docs/shell/commands/math_arccos/) — Returns the arccosine of the number.
- [`math arccosh`](/docs/shell/commands/math_arccosh/) — Returns the inverse of the hyperbolic cosine function.
- [`math arcsin`](/docs/shell/commands/math_arcsin/) — Returns the arcsine of the number.
- [`math arcsinh`](/docs/shell/commands/math_arcsinh/) — Returns the inverse of the hyperbolic sine function.
- [`math arctan`](/docs/shell/commands/math_arctan/) — Returns the arctangent of the number.
- [`math arctanh`](/docs/shell/commands/math_arctanh/) — Returns the inverse of the hyperbolic tangent function.
- [`math avg`](/docs/shell/commands/math_avg/) — Returns the average of a list of numbers.
- [`math cbrt`](/docs/shell/commands/math_cbrt/) — Returns the real-valued cube root of the input number.
- [`math ceil`](/docs/shell/commands/math_ceil/) — Returns the ceil of a number (smallest integer greater than or equal to that number).
- [`math cos`](/docs/shell/commands/math_cos/) — Returns the cosine of the number.
- [`math cosh`](/docs/shell/commands/math_cosh/) — Returns the hyperbolic cosine of the number.
- [`math exp`](/docs/shell/commands/math_exp/) — Returns e raised to the power of x.
- [`math floor`](/docs/shell/commands/math_floor/) — Returns the floor of a number (largest integer less than or equal to that number).
- [`math ln`](/docs/shell/commands/math_ln/) — Returns the natural logarithm. Base: (math e).
- [`math log`](/docs/shell/commands/math_log/) — Returns the logarithm for an arbitrary base.
- [`math max`](/docs/shell/commands/math_max/) — Returns the maximum of a list of values, or of columns in a table.
- [`math median`](/docs/shell/commands/math_median/) — Computes the median of a list of numbers.
- [`math min`](/docs/shell/commands/math_min/) — Finds the minimum within a list of values or tables.
- [`math mode`](/docs/shell/commands/math_mode/) — Returns the most frequent element(s) from a list of numbers or tables.
- [`math product`](/docs/shell/commands/math_product/) — Returns the product of a list of numbers or the products of each column of a table.
- [`math round`](/docs/shell/commands/math_round/) — Returns the input number rounded to the specified precision.
- [`math sin`](/docs/shell/commands/math_sin/) — Returns the sine of the number.
- [`math sinh`](/docs/shell/commands/math_sinh/) — Returns the hyperbolic sine of the number.
- [`math sqrt`](/docs/shell/commands/math_sqrt/) — Returns the square root of the input number.
- [`math stddev`](/docs/shell/commands/math_stddev/) — Returns the standard deviation of a list of numbers, or of each column in a table.
- [`math sum`](/docs/shell/commands/math_sum/) — Returns the sum of a list of numbers or of each column in a table.
- [`math tan`](/docs/shell/commands/math_tan/) — Returns the tangent of the number.
- [`math tanh`](/docs/shell/commands/math_tanh/) — Returns the hyperbolic tangent of the number.
- [`math variance`](/docs/shell/commands/math_variance/) — Returns the variance of a list of numbers or of each column in a table.

## misc

- [`tutor`](/docs/shell/commands/tutor/) — Run the tutorial. To begin, run: tutor.

## network

- [`http`](/docs/shell/commands/http/) — Various commands for working with http methods.
- [`http delete`](/docs/shell/commands/http_delete/) — Delete the specified resource.
- [`http get`](/docs/shell/commands/http_get/) — Fetch the contents from a URL.
- [`http head`](/docs/shell/commands/http_head/) — Get the headers from a URL.
- [`http options`](/docs/shell/commands/http_options/) — Requests permitted communication options for a given URL.
- [`http patch`](/docs/shell/commands/http_patch/) — Send a PATCH request to a URL with a request body.
- [`http pool`](/docs/shell/commands/http_pool/) — Configure and reset builtin http connection pool.
- [`http post`](/docs/shell/commands/http_post/) — Send a POST request to a URL with a request body.
- [`http put`](/docs/shell/commands/http_put/) — Send a PUT request to a URL with a request body.
- [`port`](/docs/shell/commands/port/) — Get a free TCP port from system.
- [`query web`](/docs/shell/commands/query_web/) — execute selector query on html/web
- [`query webpage-info`](/docs/shell/commands/query_webpage-info/) — uses the webpage crate to extract info from html: title, description, language, links, RSS feeds, Opengraph, Schema.org, and more
- [`url`](/docs/shell/commands/url/) — Various commands for working with URLs.
- [`url build-query`](/docs/shell/commands/url_build-query/) — Converts record or table into query string applying percent-encoding.
- [`url join`](/docs/shell/commands/url_join/) — Convert a record to a URL string.
- [`url parse`](/docs/shell/commands/url_parse/) — Parse a URL string into structured data.
- [`url split-query`](/docs/shell/commands/url_split-query/) — Converts query string into table applying percent-decoding.

## path

- [`path`](/docs/shell/commands/path/) — Explore and manipulate paths.
- [`path basename`](/docs/shell/commands/path_basename/) — Get the final component of a path.
- [`path dirname`](/docs/shell/commands/path_dirname/) — Get the parent directory of a path.
- [`path exists`](/docs/shell/commands/path_exists/) — Check whether a path exists.
- [`path expand`](/docs/shell/commands/path_expand/) — Try to expand a path to its absolute form.
- [`path join`](/docs/shell/commands/path_join/) — Join a structured path or a list of path parts.
- [`path parse`](/docs/shell/commands/path_parse/) — Convert a path into structured data.
- [`path relative-to`](/docs/shell/commands/path_relative-to/) — Express a path as relative to another path.
- [`path self`](/docs/shell/commands/path_self/) — Get the absolute path of the script or module containing this command at parse time.
- [`path split`](/docs/shell/commands/path_split/) — Split a path into a list based on the system's path separator.
- [`path type`](/docs/shell/commands/path_type/) — Get the type of the object a path refers to (e.g., file, dir, symlink).

## platform

- [`abbr`](/docs/shell/commands/abbr/) — Abbreviations related commands.
- [`abbr list`](/docs/shell/commands/abbr_list/) — List all defined abbreviations.
- [`ansi`](/docs/shell/commands/ansi/) — Output ANSI codes to change color and style of text.
- [`ansi gradient`](/docs/shell/commands/ansi_gradient/) — Add a color gradient (using ANSI color codes) to the given string.
- [`ansi link`](/docs/shell/commands/ansi_link/) — Add a link (using OSC 8 escape sequence) to the given string.
- [`ansi strip`](/docs/shell/commands/ansi_strip/) — Strip ANSI escape sequences from a string.
- [`clear`](/docs/shell/commands/clear/) — Clear the terminal screen.
- [`input`](/docs/shell/commands/input/) — Get input from the user via the terminal.
- [`input list`](/docs/shell/commands/input_list/) — Display an interactive list for user selection.
- [`input listen`](/docs/shell/commands/input_listen/) — Listen for user interface events.
- [`is-redirected`](/docs/shell/commands/is-redirected/) — Check if the current custom command's return value is redirected away from display.
- [`is-terminal`](/docs/shell/commands/is-terminal/) — Check if the process stdin, stdout, or stderr is attached to a terminal device.
- [`keybindings`](/docs/shell/commands/keybindings/) — Keybindings related commands.
- [`keybindings default`](/docs/shell/commands/keybindings_default/) — List default keybindings.
- [`keybindings list`](/docs/shell/commands/keybindings_list/) — List available options that can be used to create keybindings.
- [`keybindings listen`](/docs/shell/commands/keybindings_listen/) — Get input from the user.
- [`kill`](/docs/shell/commands/kill/) — Kill a process using its process ID.
- [`sleep`](/docs/shell/commands/sleep/) — Delay for a specified amount of time.
- [`term`](/docs/shell/commands/term/) — Commands for querying information about the terminal.
- [`term query`](/docs/shell/commands/term_query/) — Query the terminal for information.
- [`term size`](/docs/shell/commands/term_size/) — Returns a record containing the number of columns (width) and rows (height) of the terminal.
- [`ulimit`](/docs/shell/commands/ulimit/) — Set or get resource usage limits.
- [`umask`](/docs/shell/commands/umask/) — Get or set default file creation permissions.
- [`version check`](/docs/shell/commands/version_check/) — Checks to see if you have the latest version of nushell.
- [`whoami`](/docs/shell/commands/whoami/) — Get the current username using uutils/coreutils whoami.

## plugin

- [`plugin`](/docs/shell/commands/plugin/) — Commands for managing plugins.
- [`plugin add`](/docs/shell/commands/plugin_add/) — Add a plugin to the plugin registry file.
- [`plugin list`](/docs/shell/commands/plugin_list/) — List loaded and installed plugins.
- [`plugin rm`](/docs/shell/commands/plugin_rm/) — Remove a plugin from the plugin registry file.
- [`plugin stop`](/docs/shell/commands/plugin_stop/) — Stop an installed plugin if it was running.
- [`plugin use`](/docs/shell/commands/plugin_use/) — Load a plugin from the plugin registry file into scope.

## prompt

- [`gstat`](/docs/shell/commands/gstat/) — Get the git status of a repo

## random

- [`random`](/docs/shell/commands/random/) — Generate a random value.
- [`random binary`](/docs/shell/commands/random_binary/) — Generate random bytes.
- [`random bool`](/docs/shell/commands/random_bool/) — Generate a random boolean value.
- [`random chars`](/docs/shell/commands/random_chars/) — Generate random chars uniformly distributed over ASCII letters and numbers: a-z, A-Z and 0-9.
- [`random float`](/docs/shell/commands/random_float/) — Generate a random float within a range [min..max].
- [`random int`](/docs/shell/commands/random_int/) — Generate a random integer [min..max].
- [`random pass`](/docs/shell/commands/random_pass/) — Generate a cryptologically secure password.
- [`random uuid`](/docs/shell/commands/random_uuid/) — Generate a random uuid string of the specified version.

## removed

- [`date format`](/docs/shell/commands/date_format/) — Removed command: use `format date` instead.
- [`let-env`](/docs/shell/commands/let-env/) — `let-env FOO = ...` has been removed, use `$env.FOO = ...` instead.

## shells

- [`exit`](/docs/shell/commands/exit/) — Exit Nu.

## strings

- [`char`](/docs/shell/commands/char/) — Output special characters (e.g., 'newline').
- [`decode`](/docs/shell/commands/decode/) — Decode bytes into a string.
- [`detect`](/docs/shell/commands/detect/) — Various commands for detecting things.
- [`detect columns`](/docs/shell/commands/detect_columns/) — Attempt to automatically split text into multiple columns.
- [`detect type`](/docs/shell/commands/detect_type/) — Infer Nushell datatype from a string.
- [`encode`](/docs/shell/commands/encode/) — Encode a string into bytes.
- [`format`](/docs/shell/commands/format/) — Various commands for formatting data.
- [`format date`](/docs/shell/commands/format_date/) — Format a given date using a format string.
- [`format duration`](/docs/shell/commands/format_duration/) — Outputs duration with a specified unit of time.
- [`format filesize`](/docs/shell/commands/format_filesize/) — Converts a column of filesizes to some specified format.
- [`format pattern`](/docs/shell/commands/format_pattern/) — Format columns into a string using a simple pattern.
- [`nu-check`](/docs/shell/commands/nu-check/) — Validate and parse Nushell input content.
- [`nu-highlight`](/docs/shell/commands/nu-highlight/) — Syntax highlight the input string.
- [`parse`](/docs/shell/commands/parse/) — Parse columns from string data using a simple pattern or a supplied regular expression.
- [`print`](/docs/shell/commands/print/) — Print the given values to stdout.
- [`split`](/docs/shell/commands/split/) — Split contents across desired subcommand (like row, column) via the separator.
- [`split chars`](/docs/shell/commands/split_chars/) — Split a string into a list of characters.
- [`split column`](/docs/shell/commands/split_column/) — Split a string into multiple columns using a separator.
- [`split row`](/docs/shell/commands/split_row/) — Split a string into multiple rows using a separator.
- [`split words`](/docs/shell/commands/split_words/) — Split a string's words into separate rows.
- [`str`](/docs/shell/commands/str/) — Various commands for working with string data.
- [`str camel-case`](/docs/shell/commands/str_camel-case/) — Convert a string to camelCase.
- [`str capitalize`](/docs/shell/commands/str_capitalize/) — Capitalize the first letter of text.
- [`str contains`](/docs/shell/commands/str_contains/) — Checks if string input contains a substring.
- [`str distance`](/docs/shell/commands/str_distance/) — Compare two strings and return the edit distance/Levenshtein distance.
- [`str downcase`](/docs/shell/commands/str_downcase/) — Convert text to lowercase.
- [`str ends-with`](/docs/shell/commands/str_ends-with/) — Check if an input ends with a string.
- [`str escape-regex`](/docs/shell/commands/str_escape-regex/) — Escapes special characters in the input string with '\'.
- [`str expand`](/docs/shell/commands/str_expand/) — Generates all possible combinations defined in brace expansion syntax.
- [`str index-of`](/docs/shell/commands/str_index-of/) — Returns start index of first occurrence of string in input, or -1 if no match.
- [`str join`](/docs/shell/commands/str_join/) — Concatenate multiple strings into a single string, with an optional separator between each.
- [`str kebab-case`](/docs/shell/commands/str_kebab-case/) — Convert a string to kebab-case.
- [`str length`](/docs/shell/commands/str_length/) — Output the length of any strings in the pipeline.
- [`str lowercase`](/docs/shell/commands/str_lowercase/) — Convert text to lowercase.
- [`str pascal-case`](/docs/shell/commands/str_pascal-case/) — Convert a string to PascalCase.
- [`str replace`](/docs/shell/commands/str_replace/) — Find and replace text in the input string.
- [`str reverse`](/docs/shell/commands/str_reverse/) — Reverse every string in the pipeline.
- [`str screaming-snake-case`](/docs/shell/commands/str_screaming-snake-case/) — Convert a string to SCREAMING_SNAKE_CASE.
- [`str snake-case`](/docs/shell/commands/str_snake-case/) — Convert a string to snake_case.
- [`str starts-with`](/docs/shell/commands/str_starts-with/) — Check if an input starts with a string.
- [`str stats`](/docs/shell/commands/str_stats/) — Gather word count statistics on the text.
- [`str substring`](/docs/shell/commands/str_substring/) — Get part of a string. Note that the first character of a string is index 0.
- [`str title-case`](/docs/shell/commands/str_title-case/) — Convert a string to Title Case.
- [`str trim`](/docs/shell/commands/str_trim/) — Trim whitespace or specific character.
- [`str upcase`](/docs/shell/commands/str_upcase/) — Convert text to uppercase.
- [`str uppercase`](/docs/shell/commands/str_uppercase/) — Convert text to uppercase.
- [`url decode`](/docs/shell/commands/url_decode/) — Converts a percent-encoded web safe string to a string.
- [`url encode`](/docs/shell/commands/url_encode/) — Converts a string to a percent encoded web safe string.

## system

- [`complete`](/docs/shell/commands/complete/) — Capture the outputs and exit code from an external piped in command in a nushell table.
- [`exec`](/docs/shell/commands/exec/) — Execute a command, replacing or exiting the current process, depending on platform.
- [`ps`](/docs/shell/commands/ps/) — View information about system processes.
- [`registry`](/docs/shell/commands/registry/) — Various commands for interacting with the system registry (Windows only).
- [`registry query`](/docs/shell/commands/registry_query/) — Query the Windows registry.
- [`run-external`](/docs/shell/commands/run-external/) — Runs external command.
- [`sys`](/docs/shell/commands/sys/) — View information about the system.
- [`sys cpu`](/docs/shell/commands/sys_cpu/) — View information about the system CPUs.
- [`sys disks`](/docs/shell/commands/sys_disks/) — View information about the system disks.
- [`sys host`](/docs/shell/commands/sys_host/) — View information about the system host.
- [`sys mem`](/docs/shell/commands/sys_mem/) — View information about the system memory.
- [`sys net`](/docs/shell/commands/sys_net/) — View information about the system network interfaces.
- [`sys temp`](/docs/shell/commands/sys_temp/) — View the temperatures of system components.
- [`sys users`](/docs/shell/commands/sys_users/) — View information about the users on the system.
- [`uname`](/docs/shell/commands/uname/) — Print certain system information using uutils/coreutils uname.
- [`which`](/docs/shell/commands/which/) — Finds a program file, alias or custom command. If `application` is not provided, all deduplicated commands will be returned.

## viewers

- [`explore`](/docs/shell/commands/explore/) — Explore acts as a table pager, just like `less` does for text.
- [`explore config`](/docs/shell/commands/explore_config/) — Launch a TUI to view and edit the nushell configuration interactively.
- [`explore regex`](/docs/shell/commands/explore_regex/) — Launch a TUI to create and explore regular expressions interactively.
- [`grid`](/docs/shell/commands/grid/) — Renders the output to a textual terminal grid.
- [`table`](/docs/shell/commands/table/) — Render the table.

Ported from [nushell/nushell.github.io](https://github.com/nushell/nushell.github.io). Copyright (c) 2021 Nushell Project. MIT License.
