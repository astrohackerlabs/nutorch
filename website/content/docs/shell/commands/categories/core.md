---
title: "Core"
description: "Core"
order: 1606
section: "Commands · core"
---




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

Ported from [nushell/nushell.github.io](https://github.com/nushell/nushell.github.io). Copyright (c) 2021 Nushell Project. MIT License.
